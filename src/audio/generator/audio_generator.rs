use std::path::Path;

use super::{
    audio_recipe::AudioRecipe,
    exporters::{AudioExporter, WavExporter},
    render::AudioBuffer,
};
use crate::AppResult;

#[derive(Clone, Debug, Default)]
pub struct AudioGenerator;

impl AudioGenerator {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn samples(&self, sound: &AudioRecipe) -> Vec<i16> {
        let mut buffer = AudioBuffer::silent(sound.sample_rate(), sound.duration_seconds());

        for layer in sound.layers() {
            buffer.render_layer(layer);
        }

        for effect in sound.effects() {
            buffer.apply_effect(effect);
        }

        buffer.into_samples()
    }

    pub fn write_wav(&self, path: &Path, sound: &AudioRecipe) -> AppResult<()> {
        self.export(path, sound, &WavExporter)
    }

    pub(crate) fn export(
        &self,
        path: &Path,
        sound: &AudioRecipe,
        exporter: &impl AudioExporter,
    ) -> AppResult<()> {
        let samples = self.samples(sound);
        exporter.export(path, sound.sample_rate(), &samples)
    }
}
