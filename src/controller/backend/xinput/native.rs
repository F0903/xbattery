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

    let Some(battery) = poll_battery(slot)? else {
        return Ok(None);
    };

    Ok(Some(ControllerSnapshot {
        slot,
        packet_number: state.dwPacketNumber,
        battery,
    }))
}

fn poll_battery(slot: u32) -> AppResult<Option<BatteryReading>> {
    let mut battery = XINPUT_BATTERY_INFORMATION::default();
    let result =
        unsafe { XInputGetBatteryInformation(slot, BATTERY_DEVTYPE_GAMEPAD, &mut battery) };

    if result == ERROR_DEVICE_NOT_CONNECTED.0 {
        return Ok(None);
    }

    if result != ERROR_SUCCESS.0 {
        return Err(format!(
            "XInputGetBatteryInformation failed for slot {}: {}",
            slot, result
        )
        .into());
    }

    Ok(Some(map_battery_reading(battery)))
}

fn map_battery_reading(battery: XINPUT_BATTERY_INFORMATION) -> BatteryReading {
    let kind = map_battery_type(battery.BatteryType);
    let level = map_battery_level(battery.BatteryLevel);
    // XInput defines the level byte only for known wireless battery types.
    let charge = match kind {
        BatteryKind::Alkaline | BatteryKind::Nimh if level != BatteryLevel::Unknown => {
            BatteryCharge::Coarse(level)
        }
        _ => BatteryCharge::Unknown,
    };

    BatteryReading::new(kind, charge)
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

fn map_battery_level(value: BATTERY_LEVEL) -> BatteryLevel {
    match value {
        BATTERY_LEVEL_EMPTY => BatteryLevel::Empty,
        BATTERY_LEVEL_LOW => BatteryLevel::Low,
        BATTERY_LEVEL_MEDIUM => BatteryLevel::Medium,
        BATTERY_LEVEL_FULL => BatteryLevel::Full,
        _ => BatteryLevel::default(),
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::XboxController::{
        BATTERY_LEVEL, BATTERY_LEVEL_EMPTY, BATTERY_LEVEL_FULL, BATTERY_LEVEL_LOW,
        BATTERY_LEVEL_MEDIUM, BATTERY_TYPE, BATTERY_TYPE_ALKALINE, BATTERY_TYPE_DISCONNECTED,
        BATTERY_TYPE_NIMH, BATTERY_TYPE_UNKNOWN, BATTERY_TYPE_WIRED, XINPUT_BATTERY_INFORMATION,
    };

    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading};

    use super::map_battery_reading;

    #[test]
    fn maps_levels_only_for_known_wireless_battery_types() {
        let cases = [
            (BATTERY_LEVEL_EMPTY, BatteryLevel::Empty),
            (BATTERY_LEVEL_LOW, BatteryLevel::Low),
            (BATTERY_LEVEL_MEDIUM, BatteryLevel::Medium),
            (BATTERY_LEVEL_FULL, BatteryLevel::Full),
        ];

        for kind in [BATTERY_TYPE_ALKALINE, BATTERY_TYPE_NIMH] {
            for (raw_level, level) in cases {
                assert_eq!(
                    map_battery_reading(raw_reading(kind, raw_level)).charge,
                    BatteryCharge::Coarse(level)
                );
            }
        }
    }

    #[test]
    fn zero_level_is_not_empty_when_battery_type_has_no_valid_level() {
        let cases = [
            (BATTERY_TYPE_DISCONNECTED, BatteryKind::Disconnected),
            (BATTERY_TYPE_WIRED, BatteryKind::Wired),
            (BATTERY_TYPE_UNKNOWN, BatteryKind::Unknown),
            (BATTERY_TYPE(0xfe), BatteryKind::Unknown),
        ];

        for (raw_kind, kind) in cases {
            assert_eq!(
                map_battery_reading(raw_reading(raw_kind, BATTERY_LEVEL_EMPTY)),
                BatteryReading::new(kind, BatteryCharge::Unknown)
            );
        }
    }

    #[test]
    fn unrecognized_level_is_unknown_for_a_wireless_battery() {
        assert_eq!(
            map_battery_reading(raw_reading(BATTERY_TYPE_ALKALINE, BATTERY_LEVEL(0xff))),
            BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Unknown)
        );
    }

    fn raw_reading(
        battery_type: BATTERY_TYPE,
        battery_level: BATTERY_LEVEL,
    ) -> XINPUT_BATTERY_INFORMATION {
        XINPUT_BATTERY_INFORMATION {
            BatteryType: battery_type,
            BatteryLevel: battery_level,
        }
    }
}
