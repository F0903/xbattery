use super::{
    audio_effect::AudioEffect, audio_layer::AudioLayer, audio_segment::AudioSegment,
    waveform::Waveform,
};

pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Debug)]
pub struct AudioRecipe {
    sample_rate: u32,
    duration_seconds: f32,
    layers: Vec<AudioLayer>,
    effects: Vec<AudioEffect>,
}

impl AudioRecipe {
    pub fn with_segments_and_effects(
        sample_rate: u32,
        segments: Vec<AudioSegment>,
        effects: Vec<AudioEffect>,
    ) -> Self {
        let mut cursor_seconds = 0.0;
        let mut layers = Vec::new();

        for segment in segments {
            match segment {
                AudioSegment::Tone {
                    frequencies,
                    duration_seconds,
                    volume,
                } => {
                    layers.push(AudioLayer::new(
                        Waveform::Sine,
                        frequencies,
                        cursor_seconds,
                        duration_seconds,
                        volume,
                    ));
                    cursor_seconds += duration_seconds;
                }
                AudioSegment::Silence { duration_seconds } => {
                    cursor_seconds += duration_seconds;
                }
            }
        }

        Self::with_duration(sample_rate, cursor_seconds, layers, effects)
    }

    pub fn with_layers(
        sample_rate: u32,
        layers: Vec<AudioLayer>,
        effects: Vec<AudioEffect>,
    ) -> Self {
        let duration_seconds = layers
            .iter()
            .map(AudioLayer::end_seconds)
            .fold(0.0, f32::max);

        Self::with_duration(sample_rate, duration_seconds, layers, effects)
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        self.duration_seconds
    }

    pub(crate) fn layers(&self) -> &[AudioLayer] {
        &self.layers
    }

    pub(crate) fn effects(&self) -> &[AudioEffect] {
        &self.effects
    }

    fn with_duration(
        sample_rate: u32,
        duration_seconds: f32,
        layers: Vec<AudioLayer>,
        effects: Vec<AudioEffect>,
    ) -> Self {
        Self {
            sample_rate,
            duration_seconds,
            layers,
            effects,
        }
    }
}
