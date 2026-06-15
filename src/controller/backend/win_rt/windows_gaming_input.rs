use std::thread;

use windows::Gaming::Input::{Gamepad, GamepadVibration, RawGameController};

use crate::{AppResult, controller::rumble::RumbleStep};

use super::{GamepadReport, RawControllerReport};

pub fn raw_controller_reports() -> AppResult<Vec<RawControllerReport>> {
    let controllers = RawGameController::RawGameControllers()?;
    let mut reports = Vec::with_capacity(controllers.Size()? as usize);

    for index in 0..controllers.Size()? {
        let controller = controllers.GetAt(index)?;
        let id = controller.NonRoamableId()?.to_string();
        let display_name = controller.DisplayName()?.to_string();
        let (remaining_mwh, full_charge_mwh, percent) =
            battery_capacity(controller.TryGetBatteryReport()?);

        reports.push(RawControllerReport {
            id,
            display_name,
            vendor_id: controller.HardwareVendorId()?,
            product_id: controller.HardwareProductId()?,
            is_wireless: controller.IsWireless()?,
            remaining_mwh,
            full_charge_mwh,
            percent,
        });
    }

    Ok(reports)
}

pub fn gamepad_reports() -> AppResult<Vec<GamepadReport>> {
    let gamepads = Gamepad::Gamepads()?;
    let mut reports = Vec::with_capacity(gamepads.Size()? as usize);

    for index in 0..gamepads.Size()? {
        let gamepad = gamepads.GetAt(index)?;
        let (remaining_mwh, full_charge_mwh, percent) =
            battery_capacity(gamepad.TryGetBatteryReport()?);

        reports.push(GamepadReport {
            index,
            is_wireless: gamepad.IsWireless()?,
            remaining_mwh,
            full_charge_mwh,
            percent,
        });
    }

    Ok(reports)
}

pub fn play_rumble_on_single_gamepad(steps: &[RumbleStep]) -> AppResult<bool> {
    if steps.is_empty() {
        return Ok(false);
    }

    let gamepads = Gamepad::Gamepads()?;
    if gamepads.Size()? != 1 {
        return Ok(false);
    }

    let gamepad = gamepads.GetAt(0)?;
    for step in steps {
        if let Err(err) = gamepad.SetVibration(vibration_for_step(*step)) {
            let _ = stop_gamepad_vibration(&gamepad);
            return Err(err.into());
        }
        thread::sleep(step.duration);
    }

    stop_gamepad_vibration(&gamepad)?;
    Ok(true)
}

fn battery_capacity(
    report: windows::Devices::Power::BatteryReport,
) -> (Option<i32>, Option<i32>, Option<u8>) {
    let remaining_mwh = report
        .RemainingCapacityInMilliwattHours()
        .ok()
        .and_then(|value| value.Value().ok());
    let full_charge_mwh = report
        .FullChargeCapacityInMilliwattHours()
        .ok()
        .and_then(|value| value.Value().ok());
    let percent = match (remaining_mwh, full_charge_mwh) {
        (Some(remaining), Some(full_charge)) if full_charge > 0 => {
            let value = ((remaining as f64 / full_charge as f64) * 100.0).round();
            Some(value.clamp(0.0, 100.0) as u8)
        }
        _ => None,
    };

    (remaining_mwh, full_charge_mwh, percent)
}

fn vibration_for_step(step: RumbleStep) -> GamepadVibration {
    GamepadVibration {
        LeftMotor: rumble_value(step.low_frequency),
        RightMotor: rumble_value(step.high_frequency),
        LeftTrigger: rumble_value(step.left_trigger),
        RightTrigger: rumble_value(step.right_trigger),
    }
}

fn stop_gamepad_vibration(gamepad: &Gamepad) -> windows::core::Result<()> {
    gamepad.SetVibration(GamepadVibration::default())
}

fn rumble_value(value: f32) -> f64 {
    value.clamp(0.0, 1.0) as f64
}
