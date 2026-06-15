use crate::{
    AppResult,
    controller::{
        Controller,
        battery::BatteryReading,
        rumble::{RumbleBackend, RumbleStep, RumbleTarget},
    },
};

use super::{BackendEvent, BackendEventStream, BackendKind};

pub trait ControllerInput {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>>;
}

pub trait ControllerEventInput {
    fn start_event_stream(&self) -> AppResult<BackendEventStream>;

    fn controller_from_event(&self, event: BackendEvent) -> (Controller, bool);
}

pub trait ControllerBattery {
    fn backend_kind(&self) -> BackendKind;

    fn battery_readings(&self) -> AppResult<Vec<BatteryReading>>;
}

pub trait ControllerRumbler {
    fn rumble(
        &self,
        target: RumbleTarget,
        steps: &[RumbleStep],
    ) -> AppResult<Option<RumbleBackend>>;
}
