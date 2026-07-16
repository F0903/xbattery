use windows::Gaming::Input::{Gamepad, RawGameController};

use crate::AppResult;

use super::{GamepadReport, RawControllerReport};

pub(crate) fn raw_controller_reports() -> AppResult<Vec<RawControllerReport>> {
    let controllers = RawGameController::RawGameControllers()?;
    let mut reports = Vec::with_capacity(controllers.Size()? as usize);

    for index in 0..controllers.Size()? {
        let controller = controllers.GetAt(index)?;
        let display_name = controller.DisplayName()?.to_string();
        let (remaining_mwh, full_charge_mwh, percent) =
            battery_capacity(controller.TryGetBatteryReport()?);

        reports.push(RawControllerReport {
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

pub(crate) fn gamepad_reports() -> AppResult<Vec<GamepadReport>> {
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
