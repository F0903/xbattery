mod constants;
mod device_snapshot;
mod ffi;

pub use device_snapshot::GameInputDeviceSnapshot;
#[cfg(debug_assertions)]
pub use ffi::enumerate_gamepad_snapshots;
pub use ffi::{CallbackWatcher, start_callback_watcher};
