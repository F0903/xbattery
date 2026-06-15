use crate::controller::{
    backend::{ControllerRumbler, GameInputBackend, XInputBackend},
    rumble::{RumbleBackend, RumbleTarget},
};

use super::{
    config::ControllerRumbleConfig, pattern::BatteryWarningStage, rumbler::run_stage,
    sequence::rumble_sequence,
};

pub fn rumble_single_controller(
    config: ControllerRumbleConfig,
    warning_level: u8,
) -> crate::AppResult<RumbleBackend> {
    run_stage(
        &GameInputBackend::new(),
        RumbleTarget::SingleController,
        BatteryWarningStage::diagnostic(warning_level),
        &config,
    )?
    .ok_or("requires exactly one connected GameInput controller".into())
}

pub fn rumble_single_xinput_controller(
    config: ControllerRumbleConfig,
    warning_level: u8,
) -> crate::AppResult<u32> {
    let stage = BatteryWarningStage::diagnostic(warning_level);
    match XInputBackend::new().rumble(
        RumbleTarget::SingleController,
        &rumble_sequence(config.pattern_for_stage(stage), &config),
    )? {
        Some(RumbleBackend::XInput(slot)) => Ok(slot),
        _ => Err("requires exactly one connected XInput controller".into()),
    }
}

pub fn rumble_xinput_slot(
    slot: u32,
    config: ControllerRumbleConfig,
    warning_level: u8,
) -> crate::AppResult<()> {
    let stage = BatteryWarningStage::diagnostic(warning_level);
    match XInputBackend::new().rumble(
        RumbleTarget::XInputSlot(slot),
        &rumble_sequence(config.pattern_for_stage(stage), &config),
    )? {
        Some(RumbleBackend::XInput(_)) => Ok(()),
        _ => Err(format!("XInput slot {} is not available", slot + 1).into()),
    }
}
