//! Format-neutral boundary between audio generation and encoding.

/// Rendered, interleaved samples with their playback metadata.
///
/// Samples are normalized `f32` values. Output formats decide how to represent, quantize, and
/// package them.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedAudio {
    sample_rate: u32,
    channels: u16,
    samples: Vec<f32>,
}

impl RenderedAudio {
    pub(crate) fn mono(sample_rate: u32, samples: Vec<f32>) -> Self {
        Self {
            sample_rate,
            channels: 1,
            samples,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}
