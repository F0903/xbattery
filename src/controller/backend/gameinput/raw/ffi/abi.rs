use std::ffi::c_void;

use windows::core::{GUID, HRESULT};

use super::super::GameInputBatteryState;

#[repr(C)]
pub(super) struct IGameInput {
    pub(super) vtbl: *const IGameInputVtbl,
}

#[repr(C)]
pub(super) struct IGameInputDevice {
    pub(super) vtbl: *const IGameInputDeviceVtbl,
}

#[repr(C)]
pub(super) struct IGameInputReading {
    pub(super) vtbl: *const IGameInputReadingVtbl,
}

pub(super) type GameInputReadingCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputReading, bool)>;
pub(super) type GameInputDeviceCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, i32, i32)>;
type GameInputGuideButtonCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, bool)>;
type GameInputKeyboardLayoutCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, u32, u32)>;

pub(super) const GAMEINPUT_RUMBLE_NONE: i32 = 0x0000_0000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct GameInputRumbleParams {
    pub(super) low_frequency: f32,
    pub(super) high_frequency: f32,
    pub(super) left_trigger: f32,
    pub(super) right_trigger: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct GameInputDeviceInfoPrefix {
    pub(super) info_size: u32,
    pub(super) vendor_id: u16,
    pub(super) product_id: u16,
    pub(super) revision_number: u16,
    pub(super) interface_number: u8,
    pub(super) collection_number: u8,
    pub(super) usage: GameInputUsage,
    pub(super) hardware_version: GameInputVersion,
    pub(super) firmware_version: GameInputVersion,
    pub(super) device_id: [u8; 32],
    pub(super) device_root_id: [u8; 32],
    pub(super) device_family: i32,
    pub(super) capabilities: i32,
    pub(super) supported_input: i32,
    pub(super) supported_rumble_motors: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct GameInputUsage {
    pub(super) page: u16,
    pub(super) id: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct GameInputVersion {
    pub(super) major: u16,
    pub(super) minor: u16,
    pub(super) build: u16,
    pub(super) revision: u16,
}

#[repr(C)]
#[allow(non_snake_case)]
pub(super) struct IGameInputVtbl {
    pub(super) QueryInterface:
        unsafe extern "system" fn(*mut IGameInput, *const GUID, *mut *mut c_void) -> HRESULT,
    pub(super) AddRef: unsafe extern "system" fn(*mut IGameInput) -> u32,
    pub(super) Release: unsafe extern "system" fn(*mut IGameInput) -> u32,
    pub(super) GetCurrentTimestamp: unsafe extern "system" fn(*mut IGameInput) -> u64,
    pub(super) GetCurrentReading: unsafe extern "system" fn(
        *mut IGameInput,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    pub(super) GetNextReading: unsafe extern "system" fn(
        *mut IGameInput,
        *mut c_void,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    pub(super) GetPreviousReading: unsafe extern "system" fn(
        *mut IGameInput,
        *mut c_void,
        i32,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    pub(super) GetTemporalReading: unsafe extern "system" fn(
        *mut IGameInput,
        u64,
        *mut IGameInputDevice,
        *mut *mut c_void,
    ) -> HRESULT,
    pub(super) RegisterReadingCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        i32,
        f32,
        *mut c_void,
        GameInputReadingCallback,
        *mut u64,
    ) -> HRESULT,
    pub(super) RegisterDeviceCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        i32,
        i32,
        i32,
        *mut c_void,
        GameInputDeviceCallback,
        *mut u64,
    ) -> HRESULT,
    pub(super) RegisterGuideButtonCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        *mut c_void,
        GameInputGuideButtonCallback,
        *mut u64,
    ) -> HRESULT,
    pub(super) RegisterKeyboardLayoutCallback: unsafe extern "system" fn(
        *mut IGameInput,
        *mut IGameInputDevice,
        *mut c_void,
        GameInputKeyboardLayoutCallback,
        *mut u64,
    ) -> HRESULT,
    pub(super) StopCallback: unsafe extern "system" fn(*mut IGameInput, u64),
    pub(super) UnregisterCallback: unsafe extern "system" fn(*mut IGameInput, u64, u64) -> bool,
}

#[repr(C)]
#[allow(non_snake_case)]
pub(super) struct IGameInputDeviceVtbl {
    pub(super) QueryInterface:
        unsafe extern "system" fn(*mut IGameInputDevice, *const GUID, *mut *mut c_void) -> HRESULT,
    pub(super) AddRef: unsafe extern "system" fn(*mut IGameInputDevice) -> u32,
    pub(super) Release: unsafe extern "system" fn(*mut IGameInputDevice) -> u32,
    pub(super) GetDeviceInfo: unsafe extern "system" fn(*mut IGameInputDevice) -> *const c_void,
    pub(super) GetDeviceStatus: unsafe extern "system" fn(*mut IGameInputDevice) -> i32,
    pub(super) GetBatteryState:
        unsafe extern "system" fn(*mut IGameInputDevice, *mut GameInputBatteryState),
    pub(super) CreateForceFeedbackEffect: unsafe extern "system" fn(
        *mut IGameInputDevice,
        u32,
        *const c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    pub(super) IsForceFeedbackMotorPoweredOn:
        unsafe extern "system" fn(*mut IGameInputDevice, u32) -> bool,
    pub(super) SetForceFeedbackMotorGain:
        unsafe extern "system" fn(*mut IGameInputDevice, u32, f32),
    pub(super) SetHapticMotorState:
        unsafe extern "system" fn(*mut IGameInputDevice, u32, *const c_void) -> HRESULT,
    pub(super) SetRumbleState:
        unsafe extern "system" fn(*mut IGameInputDevice, *const GameInputRumbleParams),
}

#[repr(C)]
#[allow(non_snake_case)]
pub(super) struct IGameInputReadingVtbl {
    pub(super) QueryInterface:
        unsafe extern "system" fn(*mut IGameInputReading, *const GUID, *mut *mut c_void) -> HRESULT,
    pub(super) AddRef: unsafe extern "system" fn(*mut IGameInputReading) -> u32,
    pub(super) Release: unsafe extern "system" fn(*mut IGameInputReading) -> u32,
    pub(super) GetInputKind: unsafe extern "system" fn(*mut IGameInputReading) -> i32,
    pub(super) GetTimestamp: unsafe extern "system" fn(*mut IGameInputReading) -> u64,
    pub(super) GetDevice:
        unsafe extern "system" fn(*mut IGameInputReading, *mut *mut IGameInputDevice),
}

unsafe extern "system" {
    pub(super) fn GameInputInitialize(riid: *const GUID, game_input: *mut *mut c_void) -> HRESULT;
}
