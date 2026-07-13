use super::{audio_recipe::AudioRecipe, render::AudioBuffer};
use crate::audio::rendered::RenderedAudio;

/// Renders a typed audio recipe into normalized, mono samples.
pub fn render(recipe: &AudioRecipe) -> RenderedAudio {
    let mut buffer = AudioBuffer::silent(recipe.sample_rate(), recipe.duration_seconds());

    for layer in recipe.layers() {
        buffer.render_layer(layer);
    }

    for effect in recipe.effects() {
        buffer.apply_effect(effect);
    }

    RenderedAudio::mono(recipe.sample_rate(), buffer.into_samples())
}
