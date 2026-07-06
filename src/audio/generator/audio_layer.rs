use super::waveform::Waveform;

pub const DEFAULT_ATTACK_SECONDS: f32 = 0.008;
pub const DEFAULT_DECAY_SECONDS: f32 = 0.0;
pub const DEFAULT_RELEASE_SECONDS: f32 = 0.028;
pub const DEFAULT_SUSTAIN_LEVEL: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct AudioLayer {
    waveform: Waveform,
    frequencies: Vec<f32>,
    start_seconds: f32,
    duration_seconds: f32,
    volume: f32,
    attack_seconds: f32,
    decay_seconds: f32,
    sustain_level: f32,
    release_seconds: f32,
}

impl AudioLayer {
    pub fn new(
        waveform: Waveform,
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
        waveform: Waveform,
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
        waveform: Waveform,
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
        let body = if self.attack_seconds > 0.0 && elapsed_seconds < self.attack_seconds {
            elapsed_seconds / self.attack_seconds
        } else if self.decay_seconds > 0.0 {
            let decay_progress =
                ((elapsed_seconds - self.attack_seconds).max(0.0) / self.decay_seconds).min(1.0);
            let decay_curve = 1.0 - (-5.0 * decay_progress).exp();
            1.0 - (1.0 - self.sustain_level) * decay_curve
        } else {
            1.0
        };

        let release = if self.release_seconds <= 0.0 {
            1.0
        } else {
            ((self.duration_seconds - elapsed_seconds) / self.release_seconds).min(1.0)
        };

        (body * release).clamp(0.0, 1.0)
    }
}
