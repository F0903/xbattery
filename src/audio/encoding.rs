//! Sample encodings for converting normalized render-domain values into PCM bytes.

use crate::AppResult;

/// Encodes normalized samples as signed, little-endian integer PCM.
pub trait PcmEncoding {
    /// Number of encoded bits used by each sample.
    fn bits_per_sample(&self) -> u16;

    /// Quantizes and encodes interleaved normalized samples.
    fn encode_samples(&self, samples: &[f32]) -> AppResult<Vec<u8>>;
}

/// Signed 16-bit little-endian PCM.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pcm16;

impl PcmEncoding for Pcm16 {
    fn bits_per_sample(&self) -> u16 {
        16
    }

    fn encode_samples(&self, samples: &[f32]) -> AppResult<Vec<u8>> {
        let mut bytes = sample_buffer(samples, 2)?;
        for sample in samples {
            let value = quantize_pcm16(*sample);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
}

/// Signed 24-bit little-endian PCM.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pcm24;

impl PcmEncoding for Pcm24 {
    fn bits_per_sample(&self) -> u16 {
        24
    }

    fn encode_samples(&self, samples: &[f32]) -> AppResult<Vec<u8>> {
        const MAX: i64 = (1 << 23) - 1;

        let mut bytes = sample_buffer(samples, 3)?;
        for sample in samples {
            let encoded = (quantize_signed(*sample, MAX) as i32).to_le_bytes();
            bytes.extend_from_slice(&encoded[..3]);
        }
        Ok(bytes)
    }
}

/// Signed 32-bit little-endian PCM.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pcm32;

impl PcmEncoding for Pcm32 {
    fn bits_per_sample(&self) -> u16 {
        32
    }

    fn encode_samples(&self, samples: &[f32]) -> AppResult<Vec<u8>> {
        let mut bytes = sample_buffer(samples, 4)?;
        for sample in samples {
            let value = quantize_signed(*sample, i64::from(i32::MAX)) as i32;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
}

fn sample_buffer(samples: &[f32], bytes_per_sample: usize) -> AppResult<Vec<u8>> {
    let capacity = samples
        .len()
        .checked_mul(bytes_per_sample)
        .ok_or("encoded audio is too large")?;
    Ok(Vec::with_capacity(capacity))
}

fn quantize_signed(value: f32, max: i64) -> i64 {
    (f64::from(value.clamp(-1.0, 1.0)) * max as f64).round() as i64
}

fn quantize_pcm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

#[cfg(test)]
mod tests {
    use super::{Pcm16, Pcm24, Pcm32, PcmEncoding};

    const SAMPLES: [f32; 5] = [-1.0, -0.5, 0.0, 0.5, 1.0];

    #[test]
    fn encodes_signed_pcm16_little_endian() {
        assert_eq!(
            Pcm16.encode_samples(&SAMPLES).unwrap(),
            [
                0x01, 0x80, // -32767
                0x00, 0xc0, // -16384
                0x00, 0x00, // 0
                0x00, 0x40, // 16384
                0xff, 0x7f, // 32767
            ]
        );
    }

    #[test]
    fn encodes_signed_pcm24_little_endian() {
        assert_eq!(
            Pcm24.encode_samples(&SAMPLES).unwrap(),
            [
                0x01, 0x00, 0x80, // -8388607
                0x00, 0x00, 0xc0, // -4194304
                0x00, 0x00, 0x00, // 0
                0x00, 0x00, 0x40, // 4194304
                0xff, 0xff, 0x7f, // 8388607
            ]
        );
    }

    #[test]
    fn encodes_signed_pcm32_little_endian() {
        let bytes = Pcm32.encode_samples(&[-1.0, 0.0, 1.0]).unwrap();

        assert_eq!(&bytes[0..4], (-i32::MAX).to_le_bytes());
        assert_eq!(&bytes[4..8], 0_i32.to_le_bytes());
        assert_eq!(&bytes[8..12], i32::MAX.to_le_bytes());
    }

    #[test]
    fn clips_samples_outside_the_normalized_range() {
        assert_eq!(
            Pcm16.encode_samples(&[-2.0, 2.0]).unwrap(),
            Pcm16.encode_samples(&[-1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn preserves_existing_pcm16_rounding() {
        assert_eq!(
            Pcm16.encode_samples(&[0.998_611_4]).unwrap(),
            32_722_i16.to_le_bytes()
        );
    }
}
