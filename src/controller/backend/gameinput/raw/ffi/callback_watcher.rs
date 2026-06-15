use std::{ffi::c_void, ptr, sync::mpsc};

use crate::AppResult;

use super::super::{
    GameInputEvent,
    constants::{
        GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
        GAMEINPUT_DEVICE_ANY_STATUS, GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE,
        GAMEINPUT_KIND_GAMEPAD,
    },
};
use super::{
    abi::{IGameInput, IGameInputDevice, IGameInputReading},
    game_input::GameInputHandle,
    snapshot::{snapshot_from_callback, snapshot_from_reading},
};

struct WatchContext {
    sender: mpsc::Sender<GameInputEvent>,
}

pub struct CallbackWatcher {
    game_input: GameInputHandle,
    device_token: u64,
    reading_token: u64,
    context: *mut WatchContext,
}

impl Drop for CallbackWatcher {
    fn drop(&mut self) {
        let mut can_drop_context = true;
        let game_input = self.game_input.raw();

        if !game_input.is_null() {
            unsafe {
                can_drop_context &= unregister_callback(game_input, self.reading_token);
                can_drop_context &= unregister_callback(game_input, self.device_token);
            }
        }

        if can_drop_context && !self.context.is_null() {
            unsafe {
                drop(Box::from_raw(self.context));
            }
        }
    }
}

pub fn start_callback_watcher() -> AppResult<(CallbackWatcher, mpsc::Receiver<GameInputEvent>)> {
    let game_input = GameInputHandle::new()?;
    let (sender, receiver) = mpsc::channel();
    let context = Box::into_raw(Box::new(WatchContext { sender }));
    let mut device_token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;
    let mut reading_token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;

    let device_register_result = unsafe {
        ((*(*game_input.raw()).vtbl).RegisterDeviceCallback)(
            game_input.raw(),
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            GAMEINPUT_DEVICE_ANY_STATUS,
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

    let reading_register_result = unsafe {
        ((*(*game_input.raw()).vtbl).RegisterReadingCallback)(
            game_input.raw(),
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            0.0,
            context as *mut c_void,
            Some(watch_reading_callback),
            &mut reading_token,
        )
    };

    if reading_register_result.is_err() {
        unsafe {
            unregister_callback(game_input.raw(), device_token);
            drop(Box::from_raw(context));
        }
        return Err(format!(
            "RegisterReadingCallback failed: {:?}",
            reading_register_result
        )
        .into());
    }

    Ok((
        CallbackWatcher {
            game_input,
            device_token,
            reading_token,
            context,
        },
        receiver,
    ))
}

unsafe fn unregister_callback(game_input: *mut IGameInput, token: u64) -> bool {
    if token == GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE {
        true
    } else {
        unsafe {
            ((*(*game_input).vtbl).StopCallback)(game_input, token);
            ((*(*game_input).vtbl).UnregisterCallback)(
                game_input,
                token,
                GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
            )
        }
    }
}

unsafe extern "system" fn watch_device_callback(
    _callback_token: u64,
    context: *mut c_void,
    device: *mut IGameInputDevice,
    timestamp: u64,
    current_status: i32,
    previous_status: i32,
) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WatchContext) };
    let _ = context
        .sender
        .send(GameInputEvent::device(snapshot_from_callback(
            device,
            timestamp,
            current_status,
            previous_status,
        )));
}

unsafe extern "system" fn watch_reading_callback(
    _callback_token: u64,
    context: *mut c_void,
    reading: *mut IGameInputReading,
    _has_overrun_occurred: bool,
) {
    if context.is_null() || reading.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WatchContext) };
    if let Some(snapshot) = unsafe { snapshot_from_reading(reading) } {
        let _ = context.sender.send(GameInputEvent::reading(snapshot));
    }
}
