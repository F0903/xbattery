use std::{ffi::c_void, ptr};

use crate::AppResult;

use super::super::constants::IID_IGAMEINPUT_V0;
use super::abi::{GameInputInitialize, IGameInput};

pub(super) struct GameInputHandle {
    raw: *mut IGameInput,
}

impl GameInputHandle {
    pub(super) fn new() -> AppResult<Self> {
        let mut game_input = ptr::null_mut::<c_void>();
        let create_result = unsafe { GameInputInitialize(&IID_IGAMEINPUT_V0, &mut game_input) };

        if create_result.is_err() || game_input.is_null() {
            return Err(format!("GameInputInitialize failed: {:?}", create_result).into());
        }

        Ok(Self {
            raw: game_input.cast(),
        })
    }

    pub(super) fn raw(&self) -> *mut IGameInput {
        self.raw
    }
}

impl Drop for GameInputHandle {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        unsafe {
            ((*(*self.raw).vtbl).Release)(self.raw);
        }
    }
}
