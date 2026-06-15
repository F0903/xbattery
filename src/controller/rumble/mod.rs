mod config;
mod diagnostic;
mod rumble_step;
mod rumble_target;
mod rumbler;

pub use config::{ControllerRumbleConfig, RumbleJolt, RumblePattern, RumblePatternSet};
pub use diagnostic::{
    rumble_single_controller, rumble_single_xinput_controller, rumble_xinput_slot,
};
pub use rumble_step::RumbleStep;
pub use rumble_target::RumbleTarget;
pub use rumbler::BatteryWarningRumbler;

#[cfg(test)]
mod tests;
