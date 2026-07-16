use super::super::GameInputDeviceSnapshot;

pub(super) fn snapshot_from_callback(
    _timestamp: u64,
    _current_status: i32,
    _previous_status: i32,
) -> GameInputDeviceSnapshot {
    GameInputDeviceSnapshot {
        #[cfg(debug_assertions)]
        timestamp: _timestamp,
        #[cfg(debug_assertions)]
        current_status: _current_status,
        #[cfg(debug_assertions)]
        previous_status: _previous_status,
    }
}
