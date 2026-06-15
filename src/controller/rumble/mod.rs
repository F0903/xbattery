mod backend;
mod config;
mod diagnostic;
mod pattern;
mod rumbler;
mod sequence;
mod step;
mod target;

pub use backend::RumbleBackend;
pub use config::{ControllerRumbleConfig, RumbleJolt, RumblePattern, RumblePatternSet};
pub use diagnostic::{
    rumble_single_controller, rumble_single_xinput_controller, rumble_xinput_slot,
};
pub use pattern::BatteryWarningStage;
pub use rumbler::BatteryWarningRumbler;
pub use step::RumbleStep;
pub use target::RumbleTarget;

#[cfg(test)]
mod tests;
