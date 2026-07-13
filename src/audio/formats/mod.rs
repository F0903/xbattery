//! Complete output formats built from rendered audio and optional sample encodings.

use crate::{AppResult, audio::rendered::RenderedAudio};

mod raw;
mod wav;

pub use raw::Raw;
pub use wav::Wav;

/// Converts rendered audio into a complete output byte stream.
pub trait Format {
    /// Encodes rendered audio in this format.
    fn encode(&self, audio: &RenderedAudio) -> AppResult<Vec<u8>>;
}
