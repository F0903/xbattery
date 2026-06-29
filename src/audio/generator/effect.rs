use super::sample_math;

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

pub(crate) fn low_pass(buffer: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
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

pub(crate) fn delay(
    buffer: &mut Vec<f32>,
    sample_rate: u32,
    delay_seconds: f32,
    feedback: f32,
    mix: f32,
) {
    if buffer.is_empty() || mix <= 0.0 {
        return;
    }

    let delay_samples = sample_math::sample_count(sample_rate, delay_seconds);
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

pub(crate) fn reverb(
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
    let tail_samples = sample_math::sample_count(sample_rate, room_seconds);
    let output_len = dry.len() + tail_samples;
    let mut wet = vec![0.0; output_len];
    let damping = damping.clamp(0.0, 0.99);
    let filter_alpha = 1.0 - damping;
    let mix = mix.clamp(0.0, 1.0);

    for (delay_seconds, tap_gain) in REVERB_TAPS {
        let delay_samples = sample_math::sample_count(sample_rate, *delay_seconds);
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

pub(crate) fn soft_limiter(buffer: &mut [f32], drive: f32) {
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
    let dry_peak = sample_math::peak(dry);
    let wet_peak = sample_math::peak(wet);

    if dry_peak <= f32::EPSILON || wet_peak <= f32::EPSILON {
        return 1.0;
    }

    (dry_peak * REVERB_WET_TARGET_PEAK_RATIO / wet_peak).clamp(1.0, REVERB_WET_MAKEUP_LIMIT)
}
