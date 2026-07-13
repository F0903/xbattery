use super::{audio_recipe::AudioRecipe, render::AudioBuffer, wav};
use crate::{AppResult, audio::AudioClip};

pub(crate) fn render_samples(sound: &AudioRecipe) -> Vec<i16> {
    let mut buffer = AudioBuffer::silent(sound.sample_rate(), sound.duration_seconds());

    for layer in sound.layers() {
        buffer.render_layer(layer);
    }

    for effect in sound.effects() {
        buffer.apply_effect(effect);
    }

    buffer.into_samples()
}

pub fn render_wav_clip(sound: &AudioRecipe) -> AppResult<AudioClip> {
    let samples = render_samples(sound);
    Ok(AudioClip::wav_bytes(wav::bytes(
        sound.sample_rate(),
        &samples,
    )?))
}
