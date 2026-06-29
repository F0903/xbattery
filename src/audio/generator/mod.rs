mod audio_buffer;
mod audio_generator;
mod effect;
mod envelope;
mod generated_sound;
mod generated_sound_effect;
mod generated_sound_layer;
mod generated_sound_segment;
mod generated_sound_waveform;
mod sample_math;
mod wav;

pub use audio_generator::AudioGenerator;
pub use generated_sound::{DEFAULT_SAMPLE_RATE, GeneratedSound};
pub use generated_sound_effect::GeneratedSoundEffect;
pub use generated_sound_layer::GeneratedSoundLayer;
pub use generated_sound_segment::GeneratedSoundSegment;
pub use generated_sound_waveform::GeneratedSoundWaveform;

#[cfg(test)]
mod tests;
