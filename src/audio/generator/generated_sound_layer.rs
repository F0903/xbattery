use super::generated_sound_waveform::GeneratedSoundWaveform;

pub const DEFAULT_ATTACK_SECONDS: f32 = 0.008;
pub const DEFAULT_DECAY_SECONDS: f32 = 0.0;
pub const DEFAULT_RELEASE_SECONDS: f32 = 0.028;
pub const DEFAULT_SUSTAIN_LEVEL: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct GeneratedSoundLayer {
    waveform: GeneratedSoundWaveform,
    frequencies: Vec<f32>,
    start_seconds: f32,
    duration_seconds: f32,
    volume: f32,
    attack_seconds: f32,
    decay_seconds: f32,
    sustain_level: f32,
    release_seconds: f32,
}

impl GeneratedSoundLayer {
    pub fn new(
        waveform: GeneratedSoundWaveform,
        frequencies: Vec<f32>,
        start_seconds: f32,
        duration_seconds: f32,
        volume: f32,
    ) -> Self {
        Self::with_decay_envelope(
            waveform,
            frequencies,
            start_seconds,
            duration_seconds,
            volume,
            DEFAULT_ATTACK_SECONDS,
            DEFAULT_DECAY_SECONDS,
            DEFAULT_SUSTAIN_LEVEL,
            DEFAULT_RELEASE_SECONDS,
        )
    }

    pub fn with_envelope(
        waveform: GeneratedSoundWaveform,
        frequencies: Vec<f32>,
        start_seconds: f32,
        duration_seconds: f32,
        volume: f32,
        attack_seconds: f32,
        release_seconds: f32,
    ) -> Self {
        Self::with_decay_envelope(
            waveform,
            frequencies,
            start_seconds,
            duration_seconds,
            volume,
            attack_seconds,
            DEFAULT_DECAY_SECONDS,
            DEFAULT_SUSTAIN_LEVEL,
            release_seconds,
        )
    }

    pub fn with_decay_envelope(
        waveform: GeneratedSoundWaveform,
        frequencies: Vec<f32>,
        start_seconds: f32,
        duration_seconds: f32,
        volume: f32,
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
    ) -> Self {
        Self {
            waveform,
            frequencies,
            start_seconds,
            duration_seconds,
            volume,
            attack_seconds,
            decay_seconds,
            sustain_level,
            release_seconds,
        }
    }

    pub(crate) fn waveform(&self) -> GeneratedSoundWaveform {
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

    pub(crate) fn attack_seconds(&self) -> f32 {
        self.attack_seconds
    }

    pub(crate) fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }

    pub(crate) fn sustain_level(&self) -> f32 {
        self.sustain_level
    }

    pub(crate) fn release_seconds(&self) -> f32 {
        self.release_seconds
    }

    pub(crate) fn end_seconds(&self) -> f32 {
        self.start_seconds + self.duration_seconds
    }
}
