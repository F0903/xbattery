mod audio_clip;
mod generator;
mod playback;

pub use audio_clip::AudioClip;
pub(crate) use generator::note_frequency;
pub use generator::{
    AudioEffect, AudioEnvelope, AudioLayer, AudioRecipe, AudioSegment, DEFAULT_SAMPLE_RATE,
    Waveform, render_wav_clip,
};
pub use playback::{play, play_blocking};
