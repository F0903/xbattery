use super::{
    generated_sound_effect::GeneratedSoundEffect, generated_sound_layer::GeneratedSoundLayer,
    generated_sound_segment::GeneratedSoundSegment,
    generated_sound_waveform::GeneratedSoundWaveform,
};

pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Debug)]
pub struct GeneratedSound {
    sample_rate: u32,
    duration_seconds: f32,
    layers: Vec<GeneratedSoundLayer>,
    effects: Vec<GeneratedSoundEffect>,
}

impl GeneratedSound {
    pub fn new(sample_rate: u32, segments: Vec<GeneratedSoundSegment>) -> Self {
        Self::with_segments_and_effects(sample_rate, segments, Vec::new())
    }

    pub fn with_segments_and_effects(
        sample_rate: u32,
        segments: Vec<GeneratedSoundSegment>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        let mut cursor_seconds = 0.0;
        let mut layers = Vec::new();

        for segment in segments {
            match segment {
                GeneratedSoundSegment::Tone {
                    frequencies,
                    duration_seconds,
                    volume,
                } => {
                    layers.push(GeneratedSoundLayer::new(
                        GeneratedSoundWaveform::Sine,
                        frequencies,
                        cursor_seconds,
                        duration_seconds,
                        volume,
                    ));
                    cursor_seconds += duration_seconds;
                }
                GeneratedSoundSegment::Silence { duration_seconds } => {
                    cursor_seconds += duration_seconds;
                }
            }
        }

        Self::with_duration(sample_rate, cursor_seconds, layers, effects)
    }

    pub fn with_layers(
        sample_rate: u32,
        layers: Vec<GeneratedSoundLayer>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        let duration_seconds = layers
            .iter()
            .map(GeneratedSoundLayer::end_seconds)
            .fold(0.0, f32::max);

        Self::with_duration(sample_rate, duration_seconds, layers, effects)
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        self.duration_seconds
    }

    pub(crate) fn layers(&self) -> &[GeneratedSoundLayer] {
        &self.layers
    }

    pub(crate) fn effects(&self) -> &[GeneratedSoundEffect] {
        &self.effects
    }

    #[cfg(test)]
    pub(crate) fn samples(&self) -> Vec<i16> {
        super::audio_generator::AudioGenerator::new().samples(self)
    }

    fn with_duration(
        sample_rate: u32,
        duration_seconds: f32,
        layers: Vec<GeneratedSoundLayer>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        Self {
            sample_rate,
            duration_seconds,
            layers,
            effects,
        }
    }
}
