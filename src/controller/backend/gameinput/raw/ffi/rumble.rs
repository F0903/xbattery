use std::{ffi::c_void, ptr};

use crate::{AppResult, controller::rumble::RumbleStep};

use super::super::constants::{
    GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
    GAMEINPUT_DEVICE_ANY_STATUS, GAMEINPUT_DEVICE_CONNECTED,
    GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE, GAMEINPUT_KIND_GAMEPAD,
};
use super::{
    abi::{
        GAMEINPUT_RUMBLE_NONE, GameInputDeviceInfoPrefix, GameInputRumbleParams, IGameInput,
        IGameInputDevice,
    },
    game_input::GameInputHandle,
};

struct RumbleEnumerationContext {
    devices: Vec<*mut IGameInputDevice>,
}

pub fn play_rumble_on_single_gamepad(steps: &[RumbleStep]) -> AppResult<bool> {
    if steps.is_empty() {
        return Ok(false);
    }

    let game_input = GameInputHandle::new()?;
    unsafe {
        let devices = enumerate_connected_rumble_devices(game_input.raw())?;
        play_rumble_on_single_device(devices, steps)
    }
}

unsafe fn enumerate_connected_rumble_devices(
    game_input: *mut IGameInput,
) -> AppResult<Vec<*mut IGameInputDevice>> {
    let mut context = RumbleEnumerationContext {
        devices: Vec::new(),
    };
    let mut token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;
    let register_result = unsafe {
        ((*(*game_input).vtbl).RegisterDeviceCallback)(
            game_input,
            ptr::null_mut(),
            GAMEINPUT_KIND_GAMEPAD,
            GAMEINPUT_DEVICE_ANY_STATUS,
            GAMEINPUT_BLOCKING_ENUMERATION,
            &mut context as *mut RumbleEnumerationContext as *mut c_void,
            Some(rumble_enumeration_callback),
            &mut token,
        )
    };

    if register_result.is_err() {
        unsafe {
            release_devices(&mut context.devices);
        }
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

    Ok(context.devices)
}

unsafe fn play_rumble_on_single_device(
    mut devices: Vec<*mut IGameInputDevice>,
    steps: &[RumbleStep],
) -> AppResult<bool> {
    let [device] = devices.as_slice() else {
        unsafe {
            release_devices(&mut devices);
        }
        return Ok(false);
    };
    let device = *device;

    for step in steps {
        unsafe {
            ((*(*device).vtbl).SetRumbleState)(device, &rumble_params(*step));
        }
        std::thread::sleep(step.duration);
    }

    unsafe {
        ((*(*device).vtbl).SetRumbleState)(device, &GameInputRumbleParams::default());
        release_devices(&mut devices);
    }

    Ok(true)
}

unsafe extern "system" fn rumble_enumeration_callback(
    _callback_token: u64,
    context: *mut c_void,
    device: *mut IGameInputDevice,
    _timestamp: u64,
    current_status: i32,
    _previous_status: i32,
) {
    if context.is_null() || device.is_null() || current_status & GAMEINPUT_DEVICE_CONNECTED == 0 {
        return;
    }

    if !unsafe { device_supports_rumble(device) } {
        return;
    }

    let context = unsafe { &mut *(context as *mut RumbleEnumerationContext) };
    unsafe {
        ((*(*device).vtbl).AddRef)(device);
    }
    context.devices.push(device);
}

unsafe fn device_supports_rumble(device: *mut IGameInputDevice) -> bool {
    let info = unsafe { ((*(*device).vtbl).GetDeviceInfo)(device) };
    if info.is_null() {
        return false;
    }

    let info = info.cast::<GameInputDeviceInfoPrefix>();
    unsafe { (*info).supported_rumble_motors != GAMEINPUT_RUMBLE_NONE }
}

fn rumble_params(step: RumbleStep) -> GameInputRumbleParams {
    GameInputRumbleParams {
        low_frequency: rumble_value(step.low_frequency),
        high_frequency: rumble_value(step.high_frequency),
        left_trigger: rumble_value(step.left_trigger),
        right_trigger: rumble_value(step.right_trigger),
    }
}

fn rumble_value(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

unsafe fn release_devices(devices: &mut Vec<*mut IGameInputDevice>) {
    for device in devices.drain(..) {
        if !device.is_null() {
            unsafe {
                ((*(*device).vtbl).Release)(device);
            }
        }
    }
}
