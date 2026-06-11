use crate::{AppResult, battery::BatteryReading, gameinput, winrt_input, xinput};

use super::{Controller, factory::ControllerFactory};

#[derive(Clone, Debug, Default)]
pub struct ControllerPoller {
    factory: ControllerFactory,
}

impl ControllerPoller {
    pub fn new() -> Self {
        Self {
            factory: ControllerFactory::new(),
        }
    }

    pub fn poll(&self) -> AppResult<Vec<Controller>> {
        let gameinput_controllers = self.poll_gameinput_controllers().unwrap_or_default();
        if !gameinput_controllers.is_empty() {
            return Ok(gameinput_controllers);
        }

        let xinput_controllers = self.poll_xinput_controllers()?;
        let precise_controllers = self.poll_precise_winrt_controllers().unwrap_or_default();

        if precise_controllers.is_empty() {
            Ok(xinput_controllers)
        } else if xinput_controllers.is_empty()
            || precise_controllers.len() == xinput_controllers.len()
        {
            Ok(precise_controllers)
        } else {
            Ok(xinput_controllers)
        }
    }

    pub fn from_gameinput_event(&self, snapshot: gameinput::GameInputDeviceSnapshot) -> Controller {
        self.factory
            .from_gameinput_snapshot(snapshot, self.single_xinput_battery())
    }

    pub fn single_xinput_battery(&self) -> Option<BatteryReading> {
        let batteries = self.xinput_batteries().ok()?;

        match batteries.as_slice() {
            [battery] => Some(*battery),
            _ => None,
        }
    }

    fn poll_gameinput_controllers(&self) -> AppResult<Vec<Controller>> {
        let gameinput_snapshots = gameinput::enumerate_gamepad_snapshots()?
            .into_iter()
            .filter(|snapshot| snapshot.is_connected())
            .collect::<Vec<_>>();
        let xinput_batteries = self.xinput_batteries().unwrap_or_default();
        let can_fallback_to_xinput_battery = xinput_batteries.len() == gameinput_snapshots.len();

        Ok(gameinput_snapshots
            .into_iter()
            .enumerate()
            .map(|(index, snapshot)| {
                let fallback = can_fallback_to_xinput_battery.then_some(xinput_batteries[index]);
                self.factory.from_gameinput_snapshot(snapshot, fallback)
            })
            .collect())
    }

    fn xinput_batteries(&self) -> AppResult<Vec<BatteryReading>> {
        Ok(xinput::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
    }

    fn poll_xinput_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(xinput::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| self.factory.from_xinput_snapshot(snapshot))
            .collect())
    }

    fn poll_precise_winrt_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(winrt_input::raw_controller_reports()?
            .into_iter()
            .filter(|report| report.percent.is_some())
            .map(|report| self.factory.from_winrt_report(report))
            .collect())
    }
}
