pub(crate) fn sample_count(sample_rate: u32, duration_seconds: f32) -> usize {
    ((sample_rate as f32 * duration_seconds).round() as usize).max(1)
}

pub(crate) fn second_to_sample(sample_rate: u32, seconds: f32) -> usize {
    (sample_rate as f32 * seconds).round() as usize
}

pub(crate) fn peak(buffer: &[f32]) -> f32 {
    buffer.iter().map(|value| value.abs()).fold(0.0, f32::max)
}
