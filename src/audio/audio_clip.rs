use std::{fmt, path::PathBuf, sync::Arc};

#[derive(Clone, Eq, PartialEq)]
pub enum AudioClip {
    File(PathBuf),
    WavBytes(Arc<[u8]>),
}

impl AudioClip {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    pub fn wav_bytes(bytes: Vec<u8>) -> Self {
        Self::WavBytes(bytes.into())
    }
}

impl fmt::Debug for AudioClip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => formatter.debug_tuple("File").field(path).finish(),
            Self::WavBytes(bytes) => formatter
                .debug_struct("WavBytes")
                .field("length", &bytes.len())
                .finish(),
        }
    }
}

impl fmt::Display for AudioClip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => write!(formatter, "{}", path.display()),
            Self::WavBytes(bytes) => write!(formatter, "generated WAV, {} bytes", bytes.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AudioClip;

    #[test]
    fn generated_clip_debug_output_omits_audio_payload() {
        let clip = AudioClip::wav_bytes(vec![0x52, 0x49, 0x46, 0x46]);

        assert_eq!(format!("{clip:?}"), "WavBytes { length: 4 }");
        assert_eq!(clip.to_string(), "generated WAV, 4 bytes");
    }
}
