#[derive(Clone, Debug)]
pub enum GeneratedSoundEffect {
    LowPass {
        cutoff_hz: f32,
    },
    Delay {
        delay_seconds: f32,
        feedback: f32,
        mix: f32,
    },
    Reverb {
        room_seconds: f32,
        damping: f32,
        mix: f32,
    },
    SoftLimiter {
        drive: f32,
    },
}
