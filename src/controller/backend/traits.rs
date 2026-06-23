use crate::{
    AppResult,
    controller::{Controller, battery::BatteryReading},
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
        let Ok(readings) = self.settled_battery_readings() else {
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

#[cfg(test)]
mod tests {
    use crate::controller::{
        Controller, ControllerSource,
        backend::{BackendKind, BatteryBackend},
        battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
    };

    #[test]
    fn attach_to_many_uses_settled_battery_readings() {
        let backend = FakeBatteryBackend;
        let controllers = vec![Controller::new(
            "one",
            "Controller",
            ControllerSource::GameInput,
            reading(BatteryLevel::Full),
        )];

        let controllers = backend.attach_to_many(controllers);

        assert_eq!(controllers[0].battery(), reading(BatteryLevel::Medium));
    }

    struct FakeBatteryBackend;

    impl BatteryBackend for FakeBatteryBackend {
        fn backend_kind(&self) -> BackendKind {
            BackendKind::XInput
        }

        fn battery_readings(&self) -> crate::AppResult<Vec<BatteryReading>> {
            Ok(vec![reading(BatteryLevel::Empty)])
        }

        fn settled_battery_readings(&self) -> crate::AppResult<Vec<BatteryReading>> {
            Ok(vec![reading(BatteryLevel::Medium)])
        }
    }

    fn reading(level: BatteryLevel) -> BatteryReading {
        BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level))
    }
}
