use std::{fs, path::PathBuf, time::SystemTime};

use super::{
    AudioGenerator, DEFAULT_SAMPLE_RATE, GeneratedSound, GeneratedSoundEffect, GeneratedSoundLayer,
    GeneratedSoundSegment, GeneratedSoundWaveform,
};

#[test]
fn writes_wav_file() {
    let path = temp_wav_path();

    AudioGenerator::new()
        .write_wav(
            &path,
            &GeneratedSound::new(
                DEFAULT_SAMPLE_RATE,
                vec![GeneratedSoundSegment::Tone {
                    frequencies: vec![440.0],
                    duration_seconds: 0.1,
                    volume: 0.25,
                }],
            ),
        )
        .unwrap();

    let bytes = fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"RIFF"));
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");

    fs::remove_file(path).unwrap();
}

#[test]
fn writes_layered_wav_file() {
    let path = temp_wav_path();

    AudioGenerator::new()
        .write_wav(
            &path,
            &GeneratedSound::with_layers(
                DEFAULT_SAMPLE_RATE,
                vec![
                    GeneratedSoundLayer::with_decay_envelope(
                        GeneratedSoundWaveform::Sine,
                        vec![440.0],
                        0.0,
                        0.12,
                        0.18,
                        0.006,
                        0.05,
                        0.1,
                        0.04,
                    ),
                    GeneratedSoundLayer::with_decay_envelope(
                        GeneratedSoundWaveform::Triangle,
                        vec![880.0],
                        0.02,
                        0.08,
                        0.08,
                        0.004,
                        0.03,
                        0.0,
                        0.03,
                    ),
                ],
                vec![
                    GeneratedSoundEffect::LowPass { cutoff_hz: 1400.0 },
                    GeneratedSoundEffect::Delay {
                        delay_seconds: 0.04,
                        feedback: 0.15,
                        mix: 0.12,
                    },
                    GeneratedSoundEffect::Reverb {
                        room_seconds: 0.18,
                        damping: 0.45,
                        mix: 0.10,
                    },
                    GeneratedSoundEffect::SoftLimiter { drive: 1.2 },
                ],
            ),
        )
        .unwrap();

    let bytes = fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"RIFF"));
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");

    fs::remove_file(path).unwrap();
}

#[test]
fn reverb_adds_audible_tail_after_dry_sound() {
    let segments = vec![GeneratedSoundSegment::Tone {
        frequencies: vec![440.0],
        duration_seconds: 0.08,
        volume: 0.25,
    }];
    let dry_samples = GeneratedSound::new(DEFAULT_SAMPLE_RATE, segments.clone()).samples();
    let wet_samples = GeneratedSound::with_segments_and_effects(
        DEFAULT_SAMPLE_RATE,
        segments,
        vec![GeneratedSoundEffect::Reverb {
            room_seconds: 0.24,
            damping: 0.20,
            mix: 0.65,
        }],
    )
    .samples();

    let tail_peak = wet_samples[dry_samples.len()..]
        .iter()
        .map(|sample| i32::from(*sample).abs())
        .max()
        .unwrap();

    assert!(wet_samples.len() > dry_samples.len());
    assert!(tail_peak > 256);
}

fn temp_wav_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "xbattery-generated-sound-{}-{}.wav",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
