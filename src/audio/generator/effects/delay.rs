use super::super::render::sample_math;

const MAX_REPEATS: usize = 6;
const MIN_GAIN: f32 = 0.02;

pub(crate) fn apply(
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
    let repeats = repeats(feedback);
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

fn repeats(feedback: f32) -> usize {
    if feedback <= 0.0 {
        return 1;
    }

    let mut repeats = 1;
    let mut gain = feedback;

    while repeats < MAX_REPEATS && gain >= MIN_GAIN {
        repeats += 1;
        gain *= feedback;
    }

    repeats
}
