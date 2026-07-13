mod audio_effect;
mod audio_envelope;
mod audio_generator;
mod audio_layer;
mod audio_recipe;
mod audio_segment;
mod effects;
mod render;
mod wav;
mod waveform;

pub use audio_effect::AudioEffect;
pub use audio_envelope::AudioEnvelope;
pub use audio_generator::render_wav_clip;
pub use audio_layer::AudioLayer;
pub use audio_recipe::{AudioRecipe, DEFAULT_SAMPLE_RATE};
pub use audio_segment::AudioSegment;
pub use waveform::Waveform;

#[cfg(test)]
mod tests;
