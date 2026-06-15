use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct RumbleStep {
    pub low_frequency: f32,
    pub high_frequency: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub duration: Duration,
}

impl RumbleStep {
    pub fn active(
        low_frequency: f32,
        high_frequency: f32,
        left_trigger: f32,
        right_trigger: f32,
        duration: Duration,
    ) -> Self {
        Self {
            low_frequency,
            high_frequency,
            left_trigger,
            right_trigger,
            duration,
        }
    }

    pub fn gap(duration: Duration) -> Self {
        Self {
            low_frequency: 0.0,
            high_frequency: 0.0,
            left_trigger: 0.0,
            right_trigger: 0.0,
            duration,
        }
    }
}
