use std::{ffi::c_void, ptr};

use crate::AppResult;

use super::super::{
    GameInputDeviceSnapshot,
    constants::{
        GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_DEVICE_ANY_STATUS,
        GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE, GAMEINPUT_KIND_GAMEPAD,
    },
};
use super::{
    abi::{IGameInput, IGameInputDevice},
    callback_registration::unregister_callback,
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
    let context = Box::into_raw(Box::new(EnumerationContext {
        snapshots: Vec::new(),
    }));
    let mut token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;
    let register_result = unsafe {
        ((*(*game_input).vtbl).RegisterDeviceCallback)(
            game_input,
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            GAMEINPUT_DEVICE_ANY_STATUS,
            GAMEINPUT_BLOCKING_ENUMERATION,
            context.cast::<c_void>(),
            Some(enumeration_callback),
            &mut token,
        )
    };

    if register_result.is_err() {
        unsafe {
            drop(Box::from_raw(context));
        }
        return Err(format!("RegisterDeviceCallback failed: {:?}", register_result).into());
    }

    if !unsafe { unregister_callback(game_input, token) } {
        // An in-flight callback still owns this pointer. Leaking it is the only safe option.
        return Err("UnregisterCallback timed out during GameInput enumeration".into());
    }

    let context = unsafe { Box::from_raw(context) };
    Ok(context.snapshots)
}

unsafe extern "system" fn enumeration_callback(
    _callback_token: u64,
    context: *mut c_void,
    _device: *mut IGameInputDevice,
    timestamp: u64,
    current_status: i32,
    previous_status: i32,
) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &mut *(context as *mut EnumerationContext) };
    context.snapshots.push(snapshot_from_callback(
        timestamp,
        current_status,
        previous_status,
    ));
}
