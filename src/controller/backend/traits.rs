use crate::{
    AppResult,
    controller::{
        Controller,
        battery::BatteryReading,
        rumble::{RumbleStep, RumbleTarget},
    },
};

use super::{BackendEvent, BackendEventStream, BackendKind};

pub trait InputBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>>;
}

pub trait EventBackend {
    fn start_event_stream(&self) -> AppResult<BackendEventStream>;

    fn controller_from_event(&self, event: BackendEvent) -> (Controller, bool);
}

pub trait BatteryBackend {
    fn backend_kind(&self) -> BackendKind;

    fn battery_readings(&self) -> AppResult<Vec<BatteryReading>>;

    fn settled_battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        self.battery_readings()
    }

    fn attach_to_many(&self, controllers: Vec<Controller>) -> Vec<Controller> {
        let Ok(readings) = self.battery_readings() else {
            return controllers;
        };

        if readings.len() != controllers.len() {
            return controllers;
        }

        let source = self.backend_kind();
        controllers
            .into_iter()
            .zip(readings)
            .map(|(controller, reading)| controller.with_battery(source, reading))
            .collect()
    }

    fn attach_to_one(&self, controller: Controller) -> Controller {
        let Ok(readings) = self.settled_battery_readings() else {
            return controller;
        };

        match readings.as_slice() {
            [reading] => controller.with_battery(self.backend_kind(), *reading),
            _ => controller,
        }
    }
}

pub trait RumbleBackend {
    fn rumble(&self, target: RumbleTarget, steps: &[RumbleStep]) -> AppResult<Option<BackendKind>>;
}
