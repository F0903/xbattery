use crate::{
    AppResult,
    audio::{encoding::PcmEncoding, formats::Format, rendered::RenderedAudio},
};

/// Headerless, interleaved PCM. Sample rate and channel count remain external metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct Raw<E> {
    encoding: E,
}

impl<E> Raw<E> {
    pub const fn new(encoding: E) -> Self {
        Self { encoding }
    }
}

impl<E: PcmEncoding> Format for Raw<E> {
    fn encode(&self, audio: &RenderedAudio) -> AppResult<Vec<u8>> {
        self.encoding.encode_samples(audio.samples())
    }
}

#[cfg(test)]
mod tests {
    use super::Raw;
    use crate::audio::{encoding::Pcm16, formats::Format, rendered::RenderedAudio};

    #[test]
    fn emits_headerless_sample_bytes() {
        let audio = RenderedAudio::mono(48_000, vec![-1.0, 0.0, 1.0]);

        let bytes = Raw::new(Pcm16).encode(&audio).unwrap();

        assert_eq!(bytes, [0x01, 0x80, 0x00, 0x00, 0xff, 0x7f]);
    }
}
