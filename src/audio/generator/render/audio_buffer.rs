use super::super::{audio_effect::AudioEffect, audio_layer::AudioLayer, effects};
use super::sample_math;

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

    pub(crate) fn render_layer(&mut self, layer: &AudioLayer) {
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
                value * layer.volume() * layer.amplitude_at(elapsed_seconds);
        }
    }

    pub(crate) fn apply_effect(&mut self, effect: &AudioEffect) {
        effects::apply(&mut self.values, self.sample_rate, effect);
    }

    pub(crate) fn into_samples(self) -> Vec<i16> {
        self.values.into_iter().map(sample_math::pcm_i16).collect()
    }
}
