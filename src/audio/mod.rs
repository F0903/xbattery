//! Audio pipeline: typed recipes become [`rendered::RenderedAudio`], sample encodings quantize that
//! signal, and output formats package the resulting bytes for playback or export.

mod audio_clip;
pub mod encoding;
mod engine;
pub mod formats;
pub mod generator;
mod playback;
pub mod rendered;

pub use audio_clip::AudioClip;
pub use engine::AudioEngine;
pub use playback::{play, play_blocking};
