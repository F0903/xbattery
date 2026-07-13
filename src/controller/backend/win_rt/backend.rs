use crate::AppResult;

use super::{GamepadReport, RawControllerReport, windows_gaming_input};

#[derive(Clone, Copy, Debug, Default)]
pub struct WinRTBackend;

impl WinRTBackend {
    pub fn gamepad_reports(&self) -> AppResult<Vec<GamepadReport>> {
        windows_gaming_input::gamepad_reports()
    }

    pub fn raw_controller_reports(&self) -> AppResult<Vec<RawControllerReport>> {
        windows_gaming_input::raw_controller_reports()
    }
}
