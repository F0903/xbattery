use std::ffi::c_void;

use windows::core::{GUID, HRESULT};

#[repr(C)]
pub(super) struct IGameInput {
    pub(super) vtbl: *const IGameInputVtbl,
}

#[repr(C)]
pub(super) struct IGameInputDevice {
    _private: [u8; 0],
}

pub(super) type GameInputDeviceCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, i32, i32)>;
type GameInputGuideButtonCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, bool)>;
type GameInputKeyboardLayoutCallback =
    Option<unsafe extern "system" fn(u64, *mut c_void, *mut IGameInputDevice, u64, u32, u32)>;

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
    // Retain the unused v0 slot so the methods below keep their documented ABI offsets.
    pub(super) _RegisterReadingCallback: unsafe extern "system" fn(),
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
    // Retain the unused v0 slot immediately before UnregisterCallback.
    pub(super) _StopCallback: unsafe extern "system" fn(),
    pub(super) UnregisterCallback: unsafe extern "system" fn(*mut IGameInput, u64, u64) -> bool,
}

unsafe extern "system" {
    pub(super) fn GameInputInitialize(riid: *const GUID, game_input: *mut *mut c_void) -> HRESULT;
}
