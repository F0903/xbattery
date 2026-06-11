mod config;
mod diagnostic;
mod pattern;
mod rumbler;
mod sequence;

pub use config::{ControllerRumbleConfig, RumbleJolt, RumblePattern, RumblePatternSet};
pub use diagnostic::{
    rumble_single_controller, rumble_single_xinput_controller, rumble_xinput_slot,
};
pub use pattern::BatteryWarningStage;
pub use rumbler::BatteryWarningRumbler;

#[cfg(test)]
mod tests;
