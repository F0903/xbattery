use crate::battery::{BatteryCharge, BatteryKind, BatteryReading};

pub(super) const GAMEINPUT_BATTERY_UNKNOWN: i32 = -1;
pub(super) const GAMEINPUT_BATTERY_NOT_PRESENT: i32 = 0;
pub(super) const GAMEINPUT_BATTERY_DISCHARGING: i32 = 1;
pub(super) const GAMEINPUT_BATTERY_IDLE: i32 = 2;
pub(super) const GAMEINPUT_BATTERY_CHARGING: i32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GameInputBatteryState {
    pub charge_rate: f32,
    pub max_charge_rate: f32,
    pub remaining_capacity: f32,
    pub full_charge_capacity: f32,
    pub status: i32,
}

pub(super) fn map_battery_state(state: GameInputBatteryState) -> BatteryReading {
    let kind = match state.status {
        GAMEINPUT_BATTERY_NOT_PRESENT => BatteryKind::Wired,
        GAMEINPUT_BATTERY_UNKNOWN => BatteryKind::Unknown,
        GAMEINPUT_BATTERY_DISCHARGING | GAMEINPUT_BATTERY_IDLE | GAMEINPUT_BATTERY_CHARGING => {
            BatteryKind::Unknown
        }
        _ => BatteryKind::Unknown,
    };
    let charge = precise_percent(state)
        .map(BatteryCharge::Precise)
        .unwrap_or(BatteryCharge::Unknown);

    BatteryReading::new(kind, charge)
}

pub(super) fn battery_status_description(status: i32) -> &'static str {
    match status {
        GAMEINPUT_BATTERY_UNKNOWN => "unknown",
        GAMEINPUT_BATTERY_NOT_PRESENT => "not-present",
        GAMEINPUT_BATTERY_DISCHARGING => "discharging",
        GAMEINPUT_BATTERY_IDLE => "idle",
        GAMEINPUT_BATTERY_CHARGING => "charging",
        _ => "unrecognized",
    }
}

fn precise_percent(state: GameInputBatteryState) -> Option<u8> {
    if !state.remaining_capacity.is_finite()
        || !state.full_charge_capacity.is_finite()
        || state.full_charge_capacity <= 0.0
        || state.remaining_capacity < 0.0
    {
        return None;
    }

    let value = ((state.remaining_capacity / state.full_charge_capacity) * 100.0).round();
    Some(value.clamp(0.0, 100.0) as u8)
}
