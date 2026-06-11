use crate::{AppResult, rumble::RumbleStep};

use super::ffi;

pub fn play_on_single_gamepad(steps: &[RumbleStep]) -> AppResult<bool> {
    ffi::play_rumble_on_single_gamepad(steps)
}
