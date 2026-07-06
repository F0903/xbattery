mod delay;
mod low_pass;
mod reverb;
mod soft_limiter;

use super::audio_effect::AudioEffect;

pub(crate) fn apply(buffer: &mut Vec<f32>, sample_rate: u32, effect: &AudioEffect) {
    match effect {
        AudioEffect::LowPass { cutoff_hz } => {
            low_pass::apply(buffer, sample_rate, *cutoff_hz);
        }
        AudioEffect::Delay {
            delay_seconds,
            feedback,
            mix,
        } => delay::apply(buffer, sample_rate, *delay_seconds, *feedback, *mix),
        AudioEffect::Reverb {
            room_seconds,
            damping,
            mix,
        } => reverb::apply(buffer, sample_rate, *room_seconds, *damping, *mix),
        AudioEffect::SoftLimiter { drive } => {
            soft_limiter::apply(buffer, *drive);
        }
    }
}
