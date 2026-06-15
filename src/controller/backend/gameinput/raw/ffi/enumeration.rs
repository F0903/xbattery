use std::{ffi::c_void, ptr};

use crate::AppResult;

use super::super::{
    GameInputDeviceSnapshot,
    constants::{
        GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
        GAMEINPUT_DEVICE_ANY_STATUS, GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE,
        GAMEINPUT_KIND_GAMEPAD,
    },
};
use super::{
    abi::{IGameInput, IGameInputDevice},
    game_input::GameInputHandle,
    snapshot::snapshot_from_callback,
};

struct EnumerationContext {
    snapshots: Vec<GameInputDeviceSnapshot>,
}

pub fn enumerate_gamepad_snapshots() -> AppResult<Vec<GameInputDeviceSnapshot>> {
    let game_input = GameInputHandle::new()?;

    unsafe { enumerate_with_game_input(game_input.raw()) }
}

unsafe fn enumerate_with_game_input(
    game_input: *mut IGameInput,
) -> AppResult<Vec<GameInputDeviceSnapshot>> {
    let mut context = EnumerationContext {
        snapshots: Vec::new(),
    };
    let mut token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;
    let register_result = unsafe {
        ((*(*game_input).vtbl).RegisterDeviceCallback)(
            game_input,
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            GAMEINPUT_DEVICE_ANY_STATUS,
            GAMEINPUT_BLOCKING_ENUMERATION,
            &mut context as *mut EnumerationContext as *mut c_void,
            Some(enumeration_callback),
            &mut token,
        )
    };

    if register_result.is_err() {
        return Err(format!("RegisterDeviceCallback failed: {:?}", register_result).into());
    }

    if token != GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE {
        unsafe {
            ((*(*game_input).vtbl).UnregisterCallback)(
                game_input,
                token,
                GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
            );
        }
    }

    Ok(context.snapshots)
}

unsafe extern "system" fn enumeration_callback(
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

    let context = unsafe { &mut *(context as *mut EnumerationContext) };
    context.snapshots.push(snapshot_from_callback(
        device,
        timestamp,
        current_status,
        previous_status,
    ));
}
