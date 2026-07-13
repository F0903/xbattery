//! Recipe model and synthesis pipeline for rendering normalized audio samples.

mod audio_effect;
mod audio_envelope;
mod audio_layer;
mod audio_recipe;
mod audio_segment;
mod effects;
mod note;
mod render;
mod renderer;
mod waveform;

pub use audio_effect::AudioEffect;
pub use audio_envelope::AudioEnvelope;
pub use audio_layer::AudioLayer;
pub use audio_recipe::{AudioRecipe, DEFAULT_SAMPLE_RATE};
pub use audio_segment::AudioSegment;
pub(crate) use note::frequency as note_frequency;
pub use renderer::render;
pub use waveform::Waveform;

#[cfg(test)]
mod tests;
