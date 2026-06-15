use crate::controller::{
    backend::{BackendKind, GameInputBackend, RumbleBackend, XInputBackend},
    battery::BatteryWarningStage,
    rumble::RumbleTarget,
};

use super::{config::ControllerRumbleConfig, rumbler::run_stage};

pub fn rumble_single_controller(
    config: ControllerRumbleConfig,
    warning_level: u8,
) -> crate::AppResult<BackendKind> {
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
) -> crate::AppResult<()> {
    let stage = BatteryWarningStage::diagnostic(warning_level);
    match XInputBackend::new().rumble(
        RumbleTarget::SingleController,
        &config.steps_for_stage(stage),
    )? {
        Some(BackendKind::XInput) => Ok(()),
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
        &config.steps_for_stage(stage),
    )? {
        Some(BackendKind::XInput) => Ok(()),
        _ => Err(format!("XInput slot {} is not available", slot + 1).into()),
    }
}
