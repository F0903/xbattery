use super::{audio_envelope::AudioEnvelope, waveform::Waveform};

#[derive(Clone, Debug)]
pub struct AudioLayer {
    waveform: Waveform,
    frequencies: Vec<f32>,
    start_seconds: f32,
    duration_seconds: f32,
    volume: f32,
    envelope: AudioEnvelope,
}

impl AudioLayer {
    pub fn new(
        waveform: Waveform,
        frequencies: Vec<f32>,
        start_seconds: f32,
        duration_seconds: f32,
        volume: f32,
    ) -> Self {
        Self::with_audio_envelope(
            waveform,
            frequencies,
            start_seconds,
            duration_seconds,
            volume,
            AudioEnvelope::default(),
        )
    }

    pub fn with_audio_envelope(
        waveform: Waveform,
        frequencies: Vec<f32>,
        start_seconds: f32,
        duration_seconds: f32,
        volume: f32,
        envelope: AudioEnvelope,
    ) -> Self {
        Self {
            waveform,
            frequencies,
            start_seconds,
            duration_seconds,
            volume,
            envelope,
        }
    }

    pub(crate) fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub(crate) fn frequencies(&self) -> &[f32] {
        &self.frequencies
    }

    pub(crate) fn start_seconds(&self) -> f32 {
        self.start_seconds
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        self.duration_seconds
    }

    pub(crate) fn volume(&self) -> f32 {
        self.volume
    }

    pub(crate) fn end_seconds(&self) -> f32 {
        self.start_seconds + self.duration_seconds
    }

    pub(crate) fn amplitude_at(&self, elapsed_seconds: f32) -> f32 {
        self.envelope
            .amplitude_at(elapsed_seconds, self.duration_seconds)
    }
}
