use std::io::Write;

use crate::AppResult;

const PCM_FORMAT: u16 = 1;
const CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const FORMAT_CHUNK_SIZE: u32 = 16;
const RIFF_BYTES_WITHOUT_DATA: u32 = 36;
const WAV_HEADER_BYTES: usize = 44;

pub(crate) fn bytes(sample_rate: u32, samples: &[i16]) -> AppResult<Vec<u8>> {
    let (riff_size, data_size, total_size) = wav_sizes(samples)?;
    let mut bytes = Vec::with_capacity(total_size);
    write_wav(&mut bytes, sample_rate, samples, riff_size, data_size)?;

    Ok(bytes)
}

fn wav_sizes(samples: &[i16]) -> AppResult<(u32, u32, usize)> {
    let data_bytes = samples
        .len()
        .checked_mul(size_of::<i16>())
        .ok_or("generated sound is too large")?;
    let data_size = u32::try_from(data_bytes).map_err(|_| "WAV is too large")?;
    let riff_size = RIFF_BYTES_WITHOUT_DATA
        .checked_add(data_size)
        .ok_or("WAV is too large")?;
    let total_size = WAV_HEADER_BYTES
        .checked_add(data_bytes)
        .ok_or("WAV is too large")?;

    Ok((riff_size, data_size, total_size))
}

fn write_wav(
    writer: &mut impl Write,
    sample_rate: u32,
    samples: &[i16],
    riff_size: u32,
    data_size: u32,
) -> AppResult<()> {
    let bytes_per_frame = CHANNELS * (BITS_PER_SAMPLE / 8);
    let byte_rate = sample_rate
        .checked_mul(u32::from(bytes_per_frame))
        .ok_or("WAV sample rate is too large")?;

    writer.write_all(b"RIFF")?;
    writer.write_all(&riff_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&FORMAT_CHUNK_SIZE.to_le_bytes())?;
    writer.write_all(&PCM_FORMAT.to_le_bytes())?;
    writer.write_all(&CHANNELS.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&bytes_per_frame.to_le_bytes())?;
    writer.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    for sample in samples {
        writer.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BITS_PER_SAMPLE, CHANNELS, FORMAT_CHUNK_SIZE, PCM_FORMAT, bytes};

    #[test]
    fn writes_consistent_header_and_payload() {
        let sample_rate = 48_000;
        let samples = [i16::MIN, -1, 0, 1, i16::MAX];

        let bytes = bytes(sample_rate, &samples).unwrap();
        let expected_payload = samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>();

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32_at(&bytes, 4) as usize, bytes.len() - 8);
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(u32_at(&bytes, 16), FORMAT_CHUNK_SIZE);
        assert_eq!(u16_at(&bytes, 20), PCM_FORMAT);
        assert_eq!(u16_at(&bytes, 22), CHANNELS);
        assert_eq!(u32_at(&bytes, 24), sample_rate);
        assert_eq!(u32_at(&bytes, 28), sample_rate * 2);
        assert_eq!(u16_at(&bytes, 32), 2);
        assert_eq!(u16_at(&bytes, 34), BITS_PER_SAMPLE);
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32_at(&bytes, 40) as usize, expected_payload.len());
        assert_eq!(&bytes[44..], expected_payload);
    }

    fn u16_at(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
}
