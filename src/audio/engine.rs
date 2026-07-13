use crate::{
    AppResult,
    audio::{
        formats::Format,
        generator::{self, AudioRecipe},
        rendered::RenderedAudio,
    },
};

/// Orchestrates recipe rendering and audio-format encoding.
#[derive(Clone, Copy, Debug, Default)]
pub struct AudioEngine<F> {
    format: F,
}

impl<F: Format> AudioEngine<F> {
    pub const fn new(format: F) -> Self {
        Self { format }
    }

    /// Renders a recipe without selecting a sample encoding or container.
    pub fn render_audio(&self, recipe: &AudioRecipe) -> RenderedAudio {
        generator::render(recipe)
    }

    /// Encodes previously rendered audio in the selected format.
    pub fn encode(&self, audio: &RenderedAudio) -> AppResult<Vec<u8>> {
        self.format.encode(audio)
    }

    /// Renders a recipe and encodes the result in the selected format.
    pub fn render(&self, recipe: &AudioRecipe) -> AppResult<Vec<u8>> {
        let audio = self.render_audio(recipe);
        self.encode(&audio)
    }
}

#[cfg(test)]
mod tests {
    use super::AudioEngine;
    use crate::AppResult;
    use crate::audio::{
        encoding::{Pcm16, Pcm24},
        formats::{Format, Raw, Wav},
        generator::{AudioLayer, AudioRecipe, DEFAULT_SAMPLE_RATE, Waveform},
        rendered::RenderedAudio,
    };

    struct MarkerFormat(u8);

    impl Format for MarkerFormat {
        fn encode(&self, audio: &RenderedAudio) -> AppResult<Vec<u8>> {
            Ok(vec![self.0, audio.channels() as u8])
        }
    }

    #[test]
    fn renders_a_recipe_to_the_selected_format() {
        let recipe = recipe(0.05);

        let bytes = AudioEngine::new(Wav::new(Pcm16)).render(&recipe).unwrap();

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn supports_custom_format_implementations() {
        let bytes = AudioEngine::new(MarkerFormat(42))
            .render(&recipe(0.01))
            .unwrap();

        assert_eq!(bytes, [42, 1]);
    }

    #[test]
    fn composes_raw_pcm_bit_depths() {
        let bytes = AudioEngine::new(Raw::new(Pcm24))
            .render(&recipe(0.01))
            .unwrap();

        assert_eq!(bytes.len(), 441 * 3);
    }

    fn recipe(duration_seconds: f32) -> AudioRecipe {
        AudioRecipe::with_layers(
            DEFAULT_SAMPLE_RATE,
            vec![AudioLayer::new(
                Waveform::Sine,
                vec![440.0],
                0.0,
                duration_seconds,
                0.2,
            )],
            Vec::new(),
        )
    }
}
