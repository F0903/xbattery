use super::generated_sound_layer::GeneratedSoundLayer;

pub(crate) fn amplitude(elapsed_seconds: f32, layer: &GeneratedSoundLayer) -> f32 {
    let body = if layer.attack_seconds() > 0.0 && elapsed_seconds < layer.attack_seconds() {
        elapsed_seconds / layer.attack_seconds()
    } else if layer.decay_seconds() > 0.0 {
        let decay_progress =
            ((elapsed_seconds - layer.attack_seconds()).max(0.0) / layer.decay_seconds()).min(1.0);
        let decay_curve = 1.0 - (-5.0 * decay_progress).exp();
        1.0 - (1.0 - layer.sustain_level()) * decay_curve
    } else {
        1.0
    };

    let release = if layer.release_seconds() <= 0.0 {
        1.0
    } else {
        ((layer.duration_seconds() - elapsed_seconds) / layer.release_seconds()).min(1.0)
    };

    (body * release).clamp(0.0, 1.0)
}
