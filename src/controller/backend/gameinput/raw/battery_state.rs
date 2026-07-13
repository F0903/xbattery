use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryReading};

pub(super) const GAMEINPUT_BATTERY_UNKNOWN: i32 = -1;
pub(super) const GAMEINPUT_BATTERY_NOT_PRESENT: i32 = 0;
pub(super) const GAMEINPUT_BATTERY_DISCHARGING: i32 = 1;
pub(super) const GAMEINPUT_BATTERY_IDLE: i32 = 2;
pub(super) const GAMEINPUT_BATTERY_CHARGING: i32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GameInputBatteryState {
    pub charge_rate: f32,
    pub max_charge_rate: f32,
    pub remaining_capacity: f32,
    pub full_charge_capacity: f32,
    pub status: i32,
}

impl Default for GameInputBatteryState {
    fn default() -> Self {
        Self {
            charge_rate: 0.0,
            max_charge_rate: 0.0,
            remaining_capacity: 0.0,
            full_charge_capacity: 0.0,
            status: GAMEINPUT_BATTERY_UNKNOWN,
        }
    }
}

pub(super) fn map_battery_state(state: GameInputBatteryState) -> BatteryReading {
    match state.status {
        GAMEINPUT_BATTERY_NOT_PRESENT => {
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown)
        }
        GAMEINPUT_BATTERY_DISCHARGING | GAMEINPUT_BATTERY_IDLE | GAMEINPUT_BATTERY_CHARGING => {
            let charge = precise_percent(state)
                .map(BatteryCharge::Precise)
                .unwrap_or(BatteryCharge::Unknown);
            BatteryReading::new(BatteryKind::Unknown, charge)
        }
        _ => BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown),
    }
}

#[cfg(debug_assertions)]
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryReading};

    use super::{
        GAMEINPUT_BATTERY_DISCHARGING, GAMEINPUT_BATTERY_NOT_PRESENT, GAMEINPUT_BATTERY_UNKNOWN,
        GameInputBatteryState, map_battery_state,
    };

    #[test]
    fn default_state_is_unknown_and_matches_the_ffi_layout() {
        let state = GameInputBatteryState::default();

        assert_eq!(state.status, GAMEINPUT_BATTERY_UNKNOWN);
        assert_eq!(size_of::<GameInputBatteryState>(), 20);
        assert_eq!(
            map_battery_state(state),
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown)
        );
    }

    #[test]
    fn ignores_capacity_for_unknown_and_absent_batteries() {
        let unknown = state(GAMEINPUT_BATTERY_UNKNOWN, 10.0, 100.0);
        let absent = state(GAMEINPUT_BATTERY_NOT_PRESENT, 10.0, 100.0);

        assert_eq!(
            map_battery_state(unknown),
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown)
        );
        assert_eq!(
            map_battery_state(absent),
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown)
        );
    }

    #[test]
    fn computes_charge_only_for_an_active_battery() {
        assert_eq!(
            map_battery_state(state(GAMEINPUT_BATTERY_DISCHARGING, 10.0, 100.0)),
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Precise(10))
        );
        assert_eq!(
            map_battery_state(state(GAMEINPUT_BATTERY_DISCHARGING, 0.0, 0.0)),
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown)
        );
    }

    fn state(
        status: i32,
        remaining_capacity: f32,
        full_charge_capacity: f32,
    ) -> GameInputBatteryState {
        GameInputBatteryState {
            remaining_capacity,
            full_charge_capacity,
            status,
            ..GameInputBatteryState::default()
        }
    }
}
