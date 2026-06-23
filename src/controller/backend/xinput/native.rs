use windows::Win32::{
    Foundation::{ERROR_DEVICE_NOT_CONNECTED, ERROR_SUCCESS},
    UI::Input::XboxController::{
        BATTERY_DEVTYPE_GAMEPAD, BATTERY_LEVEL, BATTERY_LEVEL_EMPTY, BATTERY_LEVEL_FULL,
        BATTERY_LEVEL_LOW, BATTERY_LEVEL_MEDIUM, BATTERY_TYPE, BATTERY_TYPE_ALKALINE,
        BATTERY_TYPE_DISCONNECTED, BATTERY_TYPE_NIMH, BATTERY_TYPE_UNKNOWN, BATTERY_TYPE_WIRED,
        XINPUT_BATTERY_INFORMATION, XINPUT_STATE, XInputGetBatteryInformation, XInputGetState,
        XUSER_MAX_COUNT,
    },
};

use crate::{
    AppResult,
    controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
};

use super::snapshot::ControllerSnapshot;

pub fn poll_controllers() -> AppResult<[Option<ControllerSnapshot>; XUSER_MAX_COUNT as usize]> {
    let mut snapshots = [None; XUSER_MAX_COUNT as usize];

    for slot in 0..XUSER_MAX_COUNT {
        snapshots[slot as usize] = poll_controller(slot)?;
    }

    Ok(snapshots)
}

fn poll_controller(slot: u32) -> AppResult<Option<ControllerSnapshot>> {
    let mut state = XINPUT_STATE::default();
    let state_result = unsafe { XInputGetState(slot, &mut state) };

    if state_result == ERROR_DEVICE_NOT_CONNECTED.0 {
        return Ok(None);
    }

    if state_result != ERROR_SUCCESS.0 {
        return Err(format!("XInputGetState failed for slot {}: {}", slot, state_result).into());
    }

    Ok(Some(ControllerSnapshot {
        slot,
        packet_number: state.dwPacketNumber,
        battery: poll_battery(slot)?,
    }))
}

fn poll_battery(slot: u32) -> AppResult<BatteryReading> {
    let mut battery = XINPUT_BATTERY_INFORMATION::default();
    let result =
        unsafe { XInputGetBatteryInformation(slot, BATTERY_DEVTYPE_GAMEPAD, &mut battery) };

    if result != ERROR_SUCCESS.0 {
        return Err(format!(
            "XInputGetBatteryInformation failed for slot {}: {}",
            slot, result
        )
        .into());
    }

    Ok(BatteryReading::new(
        map_battery_type(battery.BatteryType),
        map_battery_level(battery.BatteryLevel)
            .map(BatteryCharge::Coarse)
            .unwrap_or(BatteryCharge::Unknown),
    ))
}

fn map_battery_type(value: BATTERY_TYPE) -> BatteryKind {
    match value {
        BATTERY_TYPE_DISCONNECTED => BatteryKind::Disconnected,
        BATTERY_TYPE_WIRED => BatteryKind::Wired,
        BATTERY_TYPE_ALKALINE => BatteryKind::Alkaline,
        BATTERY_TYPE_NIMH => BatteryKind::Nimh,
        BATTERY_TYPE_UNKNOWN => BatteryKind::Unknown,
        _ => BatteryKind::Unknown,
    }
}

fn map_battery_level(value: BATTERY_LEVEL) -> Option<BatteryLevel> {
    match value {
        BATTERY_LEVEL_EMPTY => Some(BatteryLevel::Empty),
        BATTERY_LEVEL_LOW => Some(BatteryLevel::Low),
        BATTERY_LEVEL_MEDIUM => Some(BatteryLevel::Medium),
        BATTERY_LEVEL_FULL => Some(BatteryLevel::Full),
        _ => None,
    }
}
