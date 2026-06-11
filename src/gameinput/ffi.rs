use std::{ffi::c_void, ptr, sync::mpsc};

use windows::core::{GUID, HRESULT};

use crate::AppResult;

use super::{
    GameInputBatteryState, GameInputDeviceSnapshot, GameInputEvent,
    battery_state::map_battery_state,
    constants::{
        GAMEINPUT_BLOCKING_ENUMERATION, GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
        GAMEINPUT_DEVICE_ANY_STATUS, GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE,
        GAMEINPUT_KIND_GAMEPAD, IID_IGAMEINPUT_V0,
    },
};

#[repr(C)]
struct IGameInput {
    vtbl: *const IGameInputVtbl,
}

#[repr(C)]
struct IGameInputDevice {
    vtbl: *const IGameInputDeviceVtbl,
}

#[repr(C)]
struct IGameInputReading {
    vtbl: *const IGameInputReadingVtbl,
}

type GameInputReadingCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputReading, bool)>;
type GameInputDeviceCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, i32, i32)>;
type GameInputGuideButtonCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, bool)>;
type GameInputKeyboardLayoutCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, u32, u32)>;

#[repr(C)]
#[allow(non_snake_case)]
struct IGameInputVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IGameInput, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IGameInput) -> u32,
    Release: unsafe extern "system" fn(*mut IGameInput) -> u32,
    GetCurrentTimestamp: unsafe extern "system" fn(*mut IGameInput) -> u64,
    GetCurrentReading: unsafe extern "system" fn(
        *mut IGameInput,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    GetNextReading: unsafe extern "system" fn(
        *mut IGameInput,
        *mut c_void,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    GetPreviousReading: unsafe extern "system" fn(
        *mut IGameInput,
        *mut c_void,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    GetTemporalReading: unsafe extern "system" fn(
        *mut IGameInput,
        u64,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    RegisterReadingCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        i32,
        f32,
        *mut c_void,
        GameInputReadingCallback,
        *mut u64,
    ) -> HRESULT,
    RegisterDeviceCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        i32,
        i32,
        i32,
        *mut c_void,
        GameInputDeviceCallback,
        *mut u64,
    ) -> HRESULT,
    RegisterGuideButtonCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        *mut c_void,
        GameInputGuideButtonCallback,
        *mut u64,
    ) -> HRESULT,
    RegisterKeyboardLayoutCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        *mut c_void,
        GameInputKeyboardLayoutCallback,
        *mut u64,
    ) -> HRESULT,
    StopCallback: unsafe extern "system" fn(*mut IGameInput, u64),
    UnregisterCallback: unsafe extern "system" fn(*mut IGameInput, u64, u64) -> bool,
}

#[repr(C)]
#[allow(non_snake_case)]
struct IGameInputDeviceVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IGameInputDevice, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IGameInputDevice) -> u32,
    Release: unsafe extern "system" fn(*mut IGameInputDevice) -> u32,
    GetDeviceInfo: unsafe extern "system" fn(*mut IGameInputDevice) -> *const c_void,
    GetDeviceStatus: unsafe extern "system" fn(*mut IGameInputDevice) -> i32,
    GetBatteryState: unsafe extern "system" fn(*mut IGameInputDevice, *mut GameInputBatteryState),
}

#[repr(C)]
#[allow(non_snake_case)]
struct IGameInputReadingVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IGameInputReading, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IGameInputReading) -> u32,
    Release: unsafe extern "system" fn(*mut IGameInputReading) -> u32,
    GetInputKind: unsafe extern "system" fn(*mut IGameInputReading) -> i32,
    GetTimestamp: unsafe extern "system" fn(*mut IGameInputReading) -> u64,
    GetDevice: unsafe extern "system" fn(*mut IGameInputReading, *mut *mut IGameInputDevice),
}

unsafe extern "system" {
    fn GameInputInitialize(riid: *const GUID, game_input: *mut *mut c_void) -> HRESULT;
}

struct EnumerationContext {
    snapshots: Vec<GameInputDeviceSnapshot>,
}

struct WatchContext {
    sender: mpsc::Sender<GameInputEvent>,
}

pub struct CallbackWatcher {
    game_input: *mut IGameInput,
    device_token: u64,
    reading_token: u64,
    context: *mut WatchContext,
}

impl Drop for CallbackWatcher {
    fn drop(&mut self) {
        let mut can_drop_context = true;

        if !self.game_input.is_null() {
            unsafe {
                can_drop_context &= unregister_callback(self.game_input, self.reading_token);
                can_drop_context &= unregister_callback(self.game_input, self.device_token);
            }

            unsafe {
                ((*(*self.game_input).vtbl).Release)(self.game_input);
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
    let game_input = create_game_input()?;
    let (sender, receiver) = mpsc::channel();
    let context = Box::into_raw(Box::new(WatchContext { sender }));
    let mut device_token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;
    let mut reading_token = GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE;

    let device_register_result = unsafe {
        ((*(*game_input).vtbl).RegisterDeviceCallback)(
            game_input,
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
            ((*(*game_input).vtbl).Release)(game_input);
            drop(Box::from_raw(context));
        }
        return Err(format!(
            "RegisterDeviceCallback failed: {:?}",
            device_register_result
        )
        .into());
    }

    let reading_register_result = unsafe {
        ((*(*game_input).vtbl).RegisterReadingCallback)(
            game_input,
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
            unregister_callback(game_input, device_token);
            ((*(*game_input).vtbl).Release)(game_input);
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

pub fn enumerate_gamepad_snapshots() -> AppResult<Vec<GameInputDeviceSnapshot>> {
    let game_input = create_game_input()?;

    let result = unsafe { enumerate_with_game_input(game_input) };
    unsafe {
        ((*(*game_input).vtbl).Release)(game_input);
    }

    result
}

fn create_game_input() -> AppResult<*mut IGameInput> {
    let mut game_input = ptr::null_mut::<c_void>();
    let create_result = unsafe { GameInputInitialize(&IID_IGAMEINPUT_V0, &mut game_input) };

    if create_result.is_err() || game_input.is_null() {
        return Err(format!("GameInputInitialize failed: {:?}", create_result).into());
    }

    Ok(game_input.cast())
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

unsafe fn snapshot_from_reading(
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

fn snapshot_from_callback(
    device: *mut IGameInputDevice,
    timestamp: u64,
    current_status: i32,
    previous_status: i32,
) -> GameInputDeviceSnapshot {
    let battery = if device.is_null() {
        GameInputBatteryState::default()
    } else {
        unsafe { read_battery_state(device) }
    };

    GameInputDeviceSnapshot {
        id: format!("gameinput:{:p}", device),
        name: "GameInput controller".to_string(),
        timestamp,
        current_status,
        previous_status,
        battery: map_battery_state(battery),
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
