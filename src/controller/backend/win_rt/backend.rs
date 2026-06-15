use crate::{
    AppResult,
    controller::{
        Controller, ControllerSource,
        backend::{BackendKind, BatteryBackend, InputBackend, RumbleBackend},
        battery::BatteryReading,
        rumble::{RumbleStep, RumbleTarget},
    },
};

use super::{GamepadReport, RawControllerReport, windows_gaming_input};

#[derive(Clone, Copy, Debug, Default)]
pub struct WinRTBackend;

impl WinRTBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn gamepad_reports(&self) -> AppResult<Vec<GamepadReport>> {
        windows_gaming_input::gamepad_reports()
    }

    pub fn raw_controller_reports(&self) -> AppResult<Vec<RawControllerReport>> {
        windows_gaming_input::raw_controller_reports()
    }

    fn controller_from_report(report: RawControllerReport) -> Controller {
        let battery = report.battery();

        Controller::new(
            format!("winrt:{}", report.id),
            report.display_name,
            ControllerSource::WinRT,
            battery,
        )
    }
}

impl InputBackend for WinRTBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(windows_gaming_input::raw_controller_reports()?
            .into_iter()
            .filter(|report| report.percent.is_some())
            .map(Self::controller_from_report)
            .collect())
    }
}

impl BatteryBackend for WinRTBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::WinRT
    }

    fn battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        Ok(windows_gaming_input::raw_controller_reports()?
            .into_iter()
            .filter(|report| report.percent.is_some())
            .map(|report| report.battery())
            .collect())
    }
}

impl RumbleBackend for WinRTBackend {
    fn rumble(
        &self,
        _target: RumbleTarget,
        steps: &[RumbleStep],
    ) -> AppResult<Option<BackendKind>> {
        if windows_gaming_input::play_rumble_on_single_gamepad(steps)? {
            Ok(Some(BackendKind::WinRT))
        } else {
            Ok(None)
        }
    }
}
