pub(crate) fn apply(buffer: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
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
