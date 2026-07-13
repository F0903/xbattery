use std::ptr;

use super::super::{
    GameInputBatteryState, GameInputDeviceSnapshot, battery_state::map_battery_state,
};
use super::abi::{IGameInputDevice, IGameInputReading};

pub(super) unsafe fn snapshot_from_reading(
    reading: *mut IGameInputReading,
) -> Option<GameInputDeviceSnapshot> {
    let timestamp = unsafe { ((*(*reading).vtbl).GetTimestamp)(reading) };
    let mut device = ptr::null_mut();
    unsafe {
        ((*(*reading).vtbl).GetDevice)(reading, &mut device);
    }

    if device.is_null() {
        return None;
    }

    let current_status = unsafe { ((*(*device).vtbl).GetDeviceStatus)(device) };
    let snapshot = snapshot_from_callback(device, timestamp, current_status, current_status);
    unsafe {
        ((*(*device).vtbl).Release)(device);
    }

    Some(snapshot)
}

pub(super) fn snapshot_from_callback(
    device: *mut IGameInputDevice,
    _timestamp: u64,
    current_status: i32,
    _previous_status: i32,
) -> GameInputDeviceSnapshot {
    let battery = if device.is_null() {
        GameInputBatteryState::default()
    } else {
        unsafe { read_battery_state(device) }
    };

    GameInputDeviceSnapshot {
        id: format!("gameinput:{:p}", device),
        #[cfg(debug_assertions)]
        timestamp: _timestamp,
        current_status,
        #[cfg(debug_assertions)]
        previous_status: _previous_status,
        battery: map_battery_state(battery),
        #[cfg(debug_assertions)]
        raw_battery: battery,
    }
}

unsafe fn read_battery_state(device: *mut IGameInputDevice) -> GameInputBatteryState {
    let mut state = GameInputBatteryState::default();
    unsafe {
        ((*(*device).vtbl).GetBatteryState)(device, &mut state);
    }
    state
}
