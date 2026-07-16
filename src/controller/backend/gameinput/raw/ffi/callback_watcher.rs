use std::{
    ffi::c_void,
    ptr,
    sync::mpsc::{self, Receiver},
};

use crate::AppResult;

use super::super::{
    GameInputDeviceSnapshot,
    constants::{
        GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_DEVICE_CONNECTED,
        GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE, GAMEINPUT_KIND_GAMEPAD,
    },
};
use super::{
    abi::IGameInputDevice, callback_registration::unregister_callback, game_input::GameInputHandle,
    snapshot::snapshot_from_callback,
};

struct WatchContext {
    sender: mpsc::Sender<GameInputDeviceSnapshot>,
}

pub struct CallbackWatcher {
    game_input: GameInputHandle,
    device_token: u64,
    context: *mut WatchContext,
}

impl Drop for CallbackWatcher {
    fn drop(&mut self) {
        let game_input = self.game_input.raw();
        let can_drop_context =
            game_input.is_null() || unsafe { unregister_callback(game_input, self.device_token) };

        if can_drop_context && !self.context.is_null() {
            unsafe {
                drop(Box::from_raw(self.context));
            }
        }
    }
}

pub fn start_callback_watcher() -> AppResult<(CallbackWatcher, Receiver<GameInputDeviceSnapshot>)> {
    let game_input = GameInputHandle::new()?;
    let (sender, receiver) = mpsc::channel();
    let context = Box::into_raw(Box::new(WatchContext { sender }));
    let mut device_token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;

    let device_register_result = unsafe {
        ((*(*game_input.raw()).vtbl).RegisterDeviceCallback)(
            game_input.raw(),
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            GAMEINPUT_DEVICE_CONNECTED,
            GAMEINPUT_BLOCKING_ENUMERATION,
            context as *mut c_void,
            Some(watch_device_callback),
            &mut device_token,
        )
    };

    if device_register_result.is_err() {
        unsafe {
            drop(Box::from_raw(context));
        }
        return Err(format!(
            "RegisterDeviceCallback failed: {:?}",
            device_register_result
        )
        .into());
    }

    Ok((
        CallbackWatcher {
            game_input,
            device_token,
            context,
        },
        receiver,
    ))
}

unsafe extern "system" fn watch_device_callback(
    _callback_token: u64,
    context: *mut c_void,
    device: *mut IGameInputDevice,
    timestamp: u64,
    current_status: i32,
    previous_status: i32,
) {
    if context.is_null() || device.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WatchContext) };
    let _ = context.sender.send(snapshot_from_callback(
        timestamp,
        current_status,
        previous_status,
    ));
}
