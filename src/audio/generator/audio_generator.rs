use std::path::Path;

use super::{generated_sound::GeneratedSound, wav};
use crate::AppResult;

#[derive(Clone, Debug, Default)]
pub struct AudioGenerator;

impl AudioGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn write_wav(&self, path: &Path, sound: &GeneratedSound) -> AppResult<()> {
        let samples = sound.samples();
        wav::write_pcm_wav(path, sound.sample_rate(), &samples)
    }
}
