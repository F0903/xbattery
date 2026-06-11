use crate::{battery::BatteryReading, gameinput, winrt_input, xinput};

use super::{Controller, ControllerSource};

#[derive(Clone, Copy, Debug, Default)]
pub struct ControllerFactory;

impl ControllerFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn from_gameinput_snapshot(
        &self,
        snapshot: gameinput::GameInputDeviceSnapshot,
        battery_fallback: Option<BatteryReading>,
    ) -> Controller {
        let controller = Controller::new(
            snapshot.id,
            snapshot.name,
            ControllerSource::GameInput,
            snapshot.battery,
        );

        if controller.needs_battery_fallback() {
            if let Some(battery) = battery_fallback {
                return controller
                    .with_battery(ControllerSource::GameInputWithXInputBattery, battery);
            }
        }

        controller
    }

    pub fn from_xinput_snapshot(&self, snapshot: xinput::ControllerSnapshot) -> Controller {
        Controller::new(
            format!("xinput:{}", snapshot.slot),
            snapshot.name(),
            ControllerSource::XInput,
            snapshot.battery,
        )
    }

    pub fn from_winrt_report(&self, report: winrt_input::RawControllerReport) -> Controller {
        let battery = report.battery();

        Controller::new(
            format!("winrt:{}", report.id),
            report.display_name,
            ControllerSource::Winrt,
            battery,
        )
    }
}
