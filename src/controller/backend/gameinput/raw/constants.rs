use windows::core::GUID;

pub(super) const GAMEINPUT_KIND_GAMEPAD: i32 = 0x0004_0000;
pub(super) const GAMEINPUT_DEVICE_CONNECTED: i32 = 0x0000_0001;
#[cfg(debug_assertions)]
pub(super) const GAMEINPUT_DEVICE_ANY_STATUS: i32 = 0x00ff_ffff;
pub(super) const GAMEINPUT_BLOCKING_ENUMERATION: i32 = 2;
pub(super) const GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE: u64 = 0;
pub(super) const GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US: u64 = 5_000;

pub(super) const IID_IGAMEINPUT_V0: GUID = GUID::from_u128(0x11be2a7e_4254_445a_9c09_ffc40f006918);
