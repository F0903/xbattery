use super::{
    AudioEffect, AudioEnvelope, AudioLayer, AudioRecipe, AudioSegment, DEFAULT_SAMPLE_RATE,
    Waveform, render,
};

#[test]
fn renders_layered_audio() {
    let audio = render(&AudioRecipe::with_layers(
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
    ));

    assert_eq!(audio.sample_rate(), DEFAULT_SAMPLE_RATE);
    assert_eq!(audio.channels(), 1);
    assert!(audio.samples().iter().any(|sample| *sample != 0.0));
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
    let dry_audio = render(&dry_sound);
    let wet_audio = render(&wet_sound);
    let dry_samples = dry_audio.samples();
    let wet_samples = wet_audio.samples();

    let tail_peak = wet_samples[dry_samples.len()..]
        .iter()
        .map(|sample| sample.abs())
        .reduce(f32::max)
        .unwrap();

    assert!(wet_samples.len() > dry_samples.len());
    assert!(tail_peak > 0.005);
}
