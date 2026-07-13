use std::io::Write;

use crate::{
    AppResult,
    audio::{encoding::PcmEncoding, formats::Format, rendered::RenderedAudio},
};

const PCM_FORMAT: u16 = 1;
const FORMAT_CHUNK_SIZE: u32 = 16;
const RIFF_BYTES_WITHOUT_DATA: u32 = 36;
const WAV_HEADER_BYTES: usize = 44;

/// RIFF/WAVE container using the supplied PCM sample encoding.
#[derive(Clone, Copy, Debug, Default)]
pub struct Wav<E> {
    encoding: E,
}

impl<E> Wav<E> {
    pub const fn new(encoding: E) -> Self {
        Self { encoding }
    }
}

impl<E: PcmEncoding> Format for Wav<E> {
    fn encode(&self, audio: &RenderedAudio) -> AppResult<Vec<u8>> {
        let bits_per_sample = self.encoding.bits_per_sample();
        let bytes_per_sample = bytes_per_sample(bits_per_sample)?;
        let payload = self.encoding.encode_samples(audio.samples())?;
        let expected_payload_size = audio
            .samples()
            .len()
            .checked_mul(usize::from(bytes_per_sample))
            .ok_or("WAV is too large")?;
        if payload.len() != expected_payload_size {
            return Err("PCM encoder returned an unexpected number of bytes".into());
        }

        let sizes = wav_sizes(payload.len())?;
        let mut bytes = Vec::with_capacity(sizes.total_size);
        write_wav(
            &mut bytes,
            audio,
            bits_per_sample,
            bytes_per_sample,
            &payload,
            sizes,
        )?;

        Ok(bytes)
    }
}

fn bytes_per_sample(bits_per_sample: u16) -> AppResult<u16> {
    bits_per_sample
        .checked_div(8)
        .filter(|bytes| *bytes > 0 && bits_per_sample.is_multiple_of(8))
        .ok_or_else(|| "WAV sample encoding must use a whole number of bytes".into())
}

#[derive(Clone, Copy)]
struct WavSizes {
    riff_size: u32,
    data_size: u32,
    total_size: usize,
    padding: usize,
}

fn wav_sizes(data_bytes: usize) -> AppResult<WavSizes> {
    let data_size = u32::try_from(data_bytes).map_err(|_| "WAV is too large")?;
    let padding = data_bytes % 2;
    let riff_size = RIFF_BYTES_WITHOUT_DATA
        .checked_add(data_size)
        .and_then(|size| size.checked_add(padding as u32))
        .ok_or("WAV is too large")?;
    let total_size = WAV_HEADER_BYTES
        .checked_add(data_bytes)
        .and_then(|size| size.checked_add(padding))
        .ok_or("WAV is too large")?;

    Ok(WavSizes {
        riff_size,
        data_size,
        total_size,
        padding,
    })
}

fn write_wav(
    writer: &mut impl Write,
    audio: &RenderedAudio,
    bits_per_sample: u16,
    bytes_per_sample: u16,
    payload: &[u8],
    sizes: WavSizes,
) -> AppResult<()> {
    let bytes_per_frame = audio
        .channels()
        .checked_mul(bytes_per_sample)
        .ok_or("WAV channel count is too large")?;
    let byte_rate = audio
        .sample_rate()
        .checked_mul(u32::from(bytes_per_frame))
        .ok_or("WAV sample rate is too large")?;

    writer.write_all(b"RIFF")?;
    writer.write_all(&sizes.riff_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&FORMAT_CHUNK_SIZE.to_le_bytes())?;
    writer.write_all(&PCM_FORMAT.to_le_bytes())?;
    writer.write_all(&audio.channels().to_le_bytes())?;
    writer.write_all(&audio.sample_rate().to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&bytes_per_frame.to_le_bytes())?;
    writer.write_all(&bits_per_sample.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&sizes.data_size.to_le_bytes())?;
    writer.write_all(payload)?;
    if sizes.padding != 0 {
        writer.write_all(&[0])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FORMAT_CHUNK_SIZE, Wav};
    use crate::audio::{
        encoding::{Pcm16, Pcm24},
        formats::{Format, Raw},
        rendered::RenderedAudio,
    };

    const SAMPLE_RATE: u32 = 48_000;
    const SAMPLES: [f32; 5] = [-1.0, -0.5, 0.0, 0.5, 1.0];

    #[test]
    fn writes_consistent_pcm16_header_and_payload() {
        let audio = RenderedAudio::mono(SAMPLE_RATE, SAMPLES.to_vec());
        let raw = Raw::new(Pcm16).encode(&audio).unwrap();

        let bytes = Wav::new(Pcm16).encode(&audio).unwrap();

        assert_header(&bytes, 16, 2, SAMPLE_RATE * 2, 10, 0);
        assert_eq!(&bytes[44..], raw);
    }

    #[test]
    fn writes_pcm24_metadata_payload_and_padding() {
        let audio = RenderedAudio::mono(SAMPLE_RATE, SAMPLES.to_vec());
        let raw = Raw::new(Pcm24).encode(&audio).unwrap();

        let bytes = Wav::new(Pcm24).encode(&audio).unwrap();

        assert_header(&bytes, 24, 3, SAMPLE_RATE * 3, 15, 1);
        assert_eq!(&bytes[44..59], raw);
        assert_eq!(bytes[59], 0);
    }

    fn assert_header(
        bytes: &[u8],
        bits_per_sample: u16,
        bytes_per_frame: u16,
        byte_rate: u32,
        data_size: u32,
        padding: u32,
    ) {
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            36 + data_size + padding
        );
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(
            u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            FORMAT_CHUNK_SIZE
        );
        assert_eq!(u16::from_le_bytes(bytes[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 1);
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            SAMPLE_RATE
        );
        assert_eq!(
            u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            byte_rate
        );
        assert_eq!(
            u16::from_le_bytes(bytes[32..34].try_into().unwrap()),
            bytes_per_frame
        );
        assert_eq!(
            u16::from_le_bytes(bytes[34..36].try_into().unwrap()),
            bits_per_sample
        );
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            data_size
        );
        assert_eq!(bytes.len(), 44 + data_size as usize + padding as usize);
    }
}
