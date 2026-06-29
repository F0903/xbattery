use std::path::Path;

use super::{audio_buffer::AudioBuffer, generated_sound::GeneratedSound, wav};
use crate::AppResult;

#[derive(Clone, Debug, Default)]
pub struct AudioGenerator;

impl AudioGenerator {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn samples(&self, sound: &GeneratedSound) -> Vec<i16> {
        let mut buffer = AudioBuffer::silent(sound.sample_rate(), sound.duration_seconds());

        for layer in sound.layers() {
            buffer.render_layer(layer);
        }

        for effect in sound.effects() {
            buffer.apply_effect(effect);
        }

        buffer.into_samples()
    }

    pub fn write_wav(&self, path: &Path, sound: &GeneratedSound) -> AppResult<()> {
        let samples = self.samples(sound);
        wav::write_pcm_wav(path, sound.sample_rate(), &samples)
    }
}
