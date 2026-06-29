use super::{
    effect, envelope, generated_sound_effect::GeneratedSoundEffect,
    generated_sound_layer::GeneratedSoundLayer, sample_math,
};

pub(crate) struct AudioBuffer {
    sample_rate: u32,
    values: Vec<f32>,
}

impl AudioBuffer {
    pub(crate) fn silent(sample_rate: u32, duration_seconds: f32) -> Self {
        Self {
            sample_rate,
            values: vec![0.0; sample_math::sample_count(sample_rate, duration_seconds)],
        }
    }

    pub(crate) fn render_layer(&mut self, layer: &GeneratedSoundLayer) {
        if layer.frequencies().is_empty() {
            return;
        }

        let start = sample_math::second_to_sample(self.sample_rate, layer.start_seconds());
        let count = sample_math::sample_count(self.sample_rate, layer.duration_seconds());
        let end = start + count;

        if end > self.values.len() {
            self.values.resize(end, 0.0);
        }

        for offset in 0..count {
            let elapsed_seconds = offset as f32 / self.sample_rate as f32;
            let value = layer
                .frequencies()
                .iter()
                .map(|frequency| layer.waveform().sample(frequency * elapsed_seconds))
                .sum::<f32>()
                / layer.frequencies().len() as f32;

            self.values[start + offset] +=
                value * layer.volume() * envelope::amplitude(elapsed_seconds, layer);
        }
    }

    pub(crate) fn apply_effect(&mut self, effect: &GeneratedSoundEffect) {
        match effect {
            GeneratedSoundEffect::LowPass { cutoff_hz } => {
                effect::low_pass(&mut self.values, self.sample_rate, *cutoff_hz);
            }
            GeneratedSoundEffect::Delay {
                delay_seconds,
                feedback,
                mix,
            } => effect::delay(
                &mut self.values,
                self.sample_rate,
                *delay_seconds,
                *feedback,
                *mix,
            ),
            GeneratedSoundEffect::Reverb {
                room_seconds,
                damping,
                mix,
            } => effect::reverb(
                &mut self.values,
                self.sample_rate,
                *room_seconds,
                *damping,
                *mix,
            ),
            GeneratedSoundEffect::SoftLimiter { drive } => {
                effect::soft_limiter(&mut self.values, *drive);
            }
        }
    }

    pub(crate) fn into_samples(self) -> Vec<i16> {
        self.values.into_iter().map(sample_math::pcm_i16).collect()
    }
}
