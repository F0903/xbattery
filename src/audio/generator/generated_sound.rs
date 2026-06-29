use super::{
    generated_sound_effect::GeneratedSoundEffect, generated_sound_layer::GeneratedSoundLayer,
    generated_sound_segment::GeneratedSoundSegment,
    generated_sound_waveform::GeneratedSoundWaveform,
};

pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

const MAX_DELAY_REPEATS: usize = 6;
const MIN_DELAY_GAIN: f32 = 0.02;
const REVERB_DRY_DUCK_AMOUNT: f32 = 0.28;
const REVERB_EARLY_REFLECTION_GAIN: f32 = 0.62;
const REVERB_WET_MAKEUP_LIMIT: f32 = 2.4;
const REVERB_WET_TARGET_PEAK_RATIO: f32 = 0.55;
const REVERB_TAPS: &[(f32, f32)] = &[
    (0.017, 0.72),
    (0.023, 0.63),
    (0.031, 0.54),
    (0.043, 0.46),
    (0.059, 0.36),
    (0.079, 0.28),
    (0.103, 0.21),
    (0.127, 0.16),
];

#[derive(Clone, Debug)]
pub struct GeneratedSound {
    sample_rate: u32,
    duration_seconds: f32,
    layers: Vec<GeneratedSoundLayer>,
    effects: Vec<GeneratedSoundEffect>,
}

impl GeneratedSound {
    pub fn new(sample_rate: u32, segments: Vec<GeneratedSoundSegment>) -> Self {
        Self::with_segments_and_effects(sample_rate, segments, Vec::new())
    }

    pub fn with_segments_and_effects(
        sample_rate: u32,
        segments: Vec<GeneratedSoundSegment>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        let mut cursor_seconds = 0.0;
        let mut layers = Vec::new();

        for segment in segments {
            match segment {
                GeneratedSoundSegment::Tone {
                    frequencies,
                    duration_seconds,
                    volume,
                } => {
                    layers.push(GeneratedSoundLayer::new(
                        GeneratedSoundWaveform::Sine,
                        frequencies,
                        cursor_seconds,
                        duration_seconds,
                        volume,
                    ));
                    cursor_seconds += duration_seconds;
                }
                GeneratedSoundSegment::Silence { duration_seconds } => {
                    cursor_seconds += duration_seconds;
                }
            }
        }

        Self::with_duration(sample_rate, cursor_seconds, layers, effects)
    }

    pub fn with_layers(
        sample_rate: u32,
        layers: Vec<GeneratedSoundLayer>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        let duration_seconds = layers
            .iter()
            .map(GeneratedSoundLayer::end_seconds)
            .fold(0.0, f32::max);

        Self::with_duration(sample_rate, duration_seconds, layers, effects)
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub(crate) fn samples(&self) -> Vec<i16> {
        let mut buffer = vec![0.0; sample_count(self.sample_rate, self.duration_seconds)];

        for layer in &self.layers {
            render_layer(&mut buffer, self.sample_rate, layer);
        }

        for effect in &self.effects {
            apply_effect(&mut buffer, self.sample_rate, effect);
        }

        buffer.into_iter().map(sample).collect()
    }

    fn with_duration(
        sample_rate: u32,
        duration_seconds: f32,
        layers: Vec<GeneratedSoundLayer>,
        effects: Vec<GeneratedSoundEffect>,
    ) -> Self {
        Self {
            sample_rate,
            duration_seconds,
            layers,
            effects,
        }
    }
}

fn render_layer(buffer: &mut Vec<f32>, sample_rate: u32, layer: &GeneratedSoundLayer) {
    if layer.frequencies().is_empty() {
        return;
    }

    let start = second_to_sample(sample_rate, layer.start_seconds());
    let count = sample_count(sample_rate, layer.duration_seconds());
    let end = start + count;

    if end > buffer.len() {
        buffer.resize(end, 0.0);
    }

    for offset in 0..count {
        let elapsed_seconds = offset as f32 / sample_rate as f32;
        let value = layer
            .frequencies()
            .iter()
            .map(|frequency| layer.waveform().sample(frequency * elapsed_seconds))
            .sum::<f32>()
            / layer.frequencies().len() as f32;

        let envelope = envelope(
            elapsed_seconds,
            layer.duration_seconds(),
            layer.attack_seconds(),
            layer.decay_seconds(),
            layer.sustain_level(),
            layer.release_seconds(),
        );

        buffer[start + offset] += value * layer.volume() * envelope;
    }
}

fn apply_effect(buffer: &mut Vec<f32>, sample_rate: u32, effect: &GeneratedSoundEffect) {
    match effect {
        GeneratedSoundEffect::LowPass { cutoff_hz } => {
            apply_low_pass(buffer, sample_rate, *cutoff_hz)
        }
        GeneratedSoundEffect::Delay {
            delay_seconds,
            feedback,
            mix,
        } => apply_delay(buffer, sample_rate, *delay_seconds, *feedback, *mix),
        GeneratedSoundEffect::Reverb {
            room_seconds,
            damping,
            mix,
        } => apply_reverb(buffer, sample_rate, *room_seconds, *damping, *mix),
        GeneratedSoundEffect::SoftLimiter { drive } => apply_soft_limiter(buffer, *drive),
    }
}

fn apply_low_pass(buffer: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
    if buffer.is_empty() {
        return;
    }

    let alpha = 1.0 - (-std::f32::consts::TAU * cutoff_hz / sample_rate as f32).exp();
    let mut filtered = buffer[0];

    for value in buffer {
        filtered += alpha * (*value - filtered);
        *value = filtered;
    }
}

fn apply_delay(
    buffer: &mut Vec<f32>,
    sample_rate: u32,
    delay_seconds: f32,
    feedback: f32,
    mix: f32,
) {
    if buffer.is_empty() || mix <= 0.0 {
        return;
    }

    let delay_samples = sample_count(sample_rate, delay_seconds);
    let repeats = delay_repeats(feedback);
    let dry = buffer.clone();

    buffer.resize(dry.len() + delay_samples * repeats, 0.0);

    for value in buffer.iter_mut().take(dry.len()) {
        *value *= 1.0 - mix;
    }

    let mut gain = mix;
    for repeat in 1..=repeats {
        let offset = delay_samples * repeat;
        for (index, sample) in dry.iter().enumerate() {
            buffer[index + offset] += sample * gain;
        }

        gain *= feedback;
    }
}

fn apply_reverb(
    buffer: &mut Vec<f32>,
    sample_rate: u32,
    room_seconds: f32,
    damping: f32,
    mix: f32,
) {
    if buffer.is_empty() || room_seconds <= 0.0 || mix <= 0.0 {
        return;
    }

    let dry = buffer.clone();
    let tail_samples = sample_count(sample_rate, room_seconds);
    let output_len = dry.len() + tail_samples;
    let mut wet = vec![0.0; output_len];
    let damping = damping.clamp(0.0, 0.99);
    let filter_alpha = 1.0 - damping;
    let mix = mix.clamp(0.0, 1.0);

    for (delay_seconds, tap_gain) in REVERB_TAPS {
        let delay_samples = sample_count(sample_rate, *delay_seconds);
        let feedback = reverb_feedback(room_seconds, *delay_seconds, damping);
        let mut comb = vec![0.0; output_len];
        let mut filtered = 0.0;

        for index in 0..output_len {
            let input = dry.get(index).copied().unwrap_or(0.0);
            let delayed = index
                .checked_sub(delay_samples)
                .map(|delayed_index| comb[delayed_index])
                .unwrap_or(0.0);

            filtered += filter_alpha * (delayed - filtered);
            let tail = filtered * feedback;
            let early_reflection = delayed * REVERB_EARLY_REFLECTION_GAIN * (1.0 - damping * 0.35);

            comb[index] = input + tail;
            wet[index] += (early_reflection + tail) * tap_gain;
        }
    }

    let wet_makeup = reverb_wet_makeup(&dry, &wet);
    let dry_level = 1.0 - mix * REVERB_DRY_DUCK_AMOUNT;

    buffer.resize(output_len, 0.0);
    for (index, value) in buffer.iter_mut().enumerate() {
        let dry_value = dry.get(index).copied().unwrap_or(0.0);
        *value = dry_value * dry_level + wet[index] * wet_makeup * mix;
    }
}

fn apply_soft_limiter(buffer: &mut [f32], drive: f32) {
    let scale = drive.tanh();
    if scale == 0.0 {
        return;
    }

    for value in buffer {
        *value = (*value * drive).tanh() / scale;
    }
}

fn delay_repeats(feedback: f32) -> usize {
    if feedback <= 0.0 {
        return 1;
    }

    let mut repeats = 1;
    let mut gain = feedback;

    while repeats < MAX_DELAY_REPEATS && gain >= MIN_DELAY_GAIN {
        repeats += 1;
        gain *= feedback;
    }

    repeats
}

fn reverb_feedback(room_seconds: f32, delay_seconds: f32, damping: f32) -> f32 {
    let feedback = 0.001_f32.powf(delay_seconds / room_seconds);
    (feedback * (1.0 - damping * 0.35)).clamp(0.0, 0.88)
}

fn reverb_wet_makeup(dry: &[f32], wet: &[f32]) -> f32 {
    let dry_peak = peak(dry);
    let wet_peak = peak(wet);

    if dry_peak <= f32::EPSILON || wet_peak <= f32::EPSILON {
        return 1.0;
    }

    (dry_peak * REVERB_WET_TARGET_PEAK_RATIO / wet_peak).clamp(1.0, REVERB_WET_MAKEUP_LIMIT)
}

fn peak(buffer: &[f32]) -> f32 {
    buffer.iter().map(|value| value.abs()).fold(0.0, f32::max)
}

fn sample_count(sample_rate: u32, duration_seconds: f32) -> usize {
    ((sample_rate as f32 * duration_seconds).round() as usize).max(1)
}

fn second_to_sample(sample_rate: u32, seconds: f32) -> usize {
    (sample_rate as f32 * seconds).round() as usize
}

fn envelope(
    elapsed_seconds: f32,
    duration_seconds: f32,
    attack_seconds: f32,
    decay_seconds: f32,
    sustain_level: f32,
    release_seconds: f32,
) -> f32 {
    let body = if attack_seconds > 0.0 && elapsed_seconds < attack_seconds {
        elapsed_seconds / attack_seconds
    } else if decay_seconds > 0.0 {
        let decay_progress = ((elapsed_seconds - attack_seconds).max(0.0) / decay_seconds).min(1.0);
        let decay_curve = 1.0 - (-5.0 * decay_progress).exp();
        1.0 - (1.0 - sustain_level) * decay_curve
    } else {
        1.0
    };

    let release = if release_seconds <= 0.0 {
        1.0
    } else {
        ((duration_seconds - elapsed_seconds) / release_seconds).min(1.0)
    };

    (body * release).clamp(0.0, 1.0)
}

fn sample(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}
