pub(crate) fn apply(buffer: &mut [f32], drive: f32) {
    let scale = drive.tanh();
    if scale == 0.0 {
        return;
    }

    for value in buffer {
        *value = (*value * drive).tanh() / scale;
    }
}
