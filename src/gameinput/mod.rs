mod battery_state;
mod constants;
mod device_snapshot;
mod event;
mod ffi;

pub use battery_state::GameInputBatteryState;
pub use device_snapshot::{GameInputDeviceEvent, GameInputDeviceSnapshot};
pub use event::GameInputEvent;
pub use ffi::{CallbackWatcher, enumerate_gamepad_snapshots, start_callback_watcher};

use crate::AppResult;

pub fn enumerate_gamepads_with_device_callback() -> AppResult<Vec<GameInputDeviceEvent>> {
    Ok(enumerate_gamepad_snapshots()?
        .into_iter()
        .map(|snapshot| GameInputDeviceEvent {
            timestamp: snapshot.timestamp,
            current_status: snapshot.current_status,
            previous_status: snapshot.previous_status,
        })
        .collect())
}
