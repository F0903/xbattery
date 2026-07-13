use crate::audio::AudioClip;

use super::{
    AudioEffect, AudioEnvelope, AudioLayer, AudioRecipe, AudioSegment, DEFAULT_SAMPLE_RATE,
    Waveform,
    audio_generator::{render_samples, render_wav_clip},
};

#[test]
fn generates_layered_wav() {
    let clip = render_wav_clip(&AudioRecipe::with_layers(
        DEFAULT_SAMPLE_RATE,
        vec![
            AudioLayer::with_audio_envelope(
                Waveform::Sine,
                vec![440.0],
                0.0,
                0.12,
                0.18,
                AudioEnvelope::new(0.006, 0.05, 0.1, 0.04),
            ),
            AudioLayer::with_audio_envelope(
                Waveform::Triangle,
                vec![880.0],
                0.02,
                0.08,
                0.08,
                AudioEnvelope::new(0.004, 0.03, 0.0, 0.03),
            ),
        ],
        vec![
            AudioEffect::LowPass { cutoff_hz: 1400.0 },
            AudioEffect::Delay {
                delay_seconds: 0.04,
                feedback: 0.15,
                mix: 0.12,
            },
            AudioEffect::Reverb {
                room_seconds: 0.18,
                damping: 0.45,
                mix: 0.10,
            },
            AudioEffect::SoftLimiter { drive: 1.2 },
        ],
    ))
    .unwrap();
    let AudioClip::WavBytes(bytes) = clip else {
        panic!("expected generated WAV bytes");
    };

    assert!(bytes.starts_with(b"RIFF"));
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");
}

#[test]
fn reverb_adds_audible_tail_after_dry_sound() {
    let segments = vec![AudioSegment::Tone {
        frequencies: vec![440.0],
        duration_seconds: 0.08,
        volume: 0.25,
    }];
    let dry_sound =
        AudioRecipe::with_segments_and_effects(DEFAULT_SAMPLE_RATE, segments.clone(), Vec::new());
    let wet_sound = AudioRecipe::with_segments_and_effects(
        DEFAULT_SAMPLE_RATE,
        segments,
        vec![AudioEffect::Reverb {
            room_seconds: 0.24,
            damping: 0.20,
            mix: 0.65,
        }],
    );
    let dry_samples = render_samples(&dry_sound);
    let wet_samples = render_samples(&wet_sound);

    let tail_peak = wet_samples[dry_samples.len()..]
        .iter()
        .map(|sample| i32::from(*sample).abs())
        .max()
        .unwrap();

    assert!(wet_samples.len() > dry_samples.len());
    assert!(tail_peak > 256);
}
