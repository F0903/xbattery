use super::super::render::sample_math;

const DRY_DUCK_AMOUNT: f32 = 0.28;
const EARLY_REFLECTION_GAIN: f32 = 0.62;
const WET_MAKEUP_LIMIT: f32 = 2.4;
const WET_TARGET_PEAK_RATIO: f32 = 0.55;
const TAPS: &[(f32, f32)] = &[
    (0.017, 0.72),
    (0.023, 0.63),
    (0.031, 0.54),
    (0.043, 0.46),
    (0.059, 0.36),
    (0.079, 0.28),
    (0.103, 0.21),
    (0.127, 0.16),
];

pub(crate) fn apply(
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

    for (delay_seconds, tap_gain) in TAPS {
        let delay_samples = sample_math::sample_count(sample_rate, *delay_seconds);
        let feedback = feedback(room_seconds, *delay_seconds, damping);
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
            let early_reflection = delayed * EARLY_REFLECTION_GAIN * (1.0 - damping * 0.35);

            comb[index] = input + tail;
            wet[index] += (early_reflection + tail) * tap_gain;
        }
    }

    let wet_makeup = wet_makeup(&dry, &wet);
    let dry_level = 1.0 - mix * DRY_DUCK_AMOUNT;

    buffer.resize(output_len, 0.0);
    for (index, value) in buffer.iter_mut().enumerate() {
        let dry_value = dry.get(index).copied().unwrap_or(0.0);
        *value = dry_value * dry_level + wet[index] * wet_makeup * mix;
    }
}

fn feedback(room_seconds: f32, delay_seconds: f32, damping: f32) -> f32 {
    let feedback = 0.001_f32.powf(delay_seconds / room_seconds);
    (feedback * (1.0 - damping * 0.35)).clamp(0.0, 0.88)
}

fn wet_makeup(dry: &[f32], wet: &[f32]) -> f32 {
    let dry_peak = sample_math::peak(dry);
    let wet_peak = sample_math::peak(wet);

    if dry_peak <= f32::EPSILON || wet_peak <= f32::EPSILON {
        return 1.0;
    }

    (dry_peak * WET_TARGET_PEAK_RATIO / wet_peak).clamp(1.0, WET_MAKEUP_LIMIT)
}
