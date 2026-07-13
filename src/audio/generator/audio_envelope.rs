#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioEnvelope {
    attack_seconds: f32,
    decay_seconds: f32,
    sustain_level: f32,
    release_seconds: f32,
}

impl AudioEnvelope {
    pub const DEFAULT_ATTACK_SECONDS: f32 = 0.008;
    pub const DEFAULT_DECAY_SECONDS: f32 = 0.0;
    pub const DEFAULT_RELEASE_SECONDS: f32 = 0.028;
    pub const DEFAULT_SUSTAIN_LEVEL: f32 = 1.0;

    pub const fn new(
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
    ) -> Self {
        Self {
            attack_seconds,
            decay_seconds,
            sustain_level,
            release_seconds,
        }
    }

    pub(crate) fn amplitude_at(self, elapsed_seconds: f32, duration_seconds: f32) -> f32 {
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
            ((duration_seconds - elapsed_seconds) / self.release_seconds).min(1.0)
        };

        (body * release).clamp(0.0, 1.0)
    }
}

impl Default for AudioEnvelope {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_ATTACK_SECONDS,
            Self::DEFAULT_DECAY_SECONDS,
            Self::DEFAULT_SUSTAIN_LEVEL,
            Self::DEFAULT_RELEASE_SECONDS,
        )
    }
}
