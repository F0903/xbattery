mod raw;

use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::{
    AppResult,
    battery::BatteryReading,
    controller::{Controller, ControllerSource},
    rumble::RumbleStep,
};

pub(super) use raw::{CallbackWatcher, GameInputEvent};

use super::{BackendEvent, BackendEventStream};
use super::{
    ControllerEventInput, ControllerInput, ControllerRumbler, RumbleBackend, RumbleTarget,
};

#[derive(Clone, Debug)]
pub struct GameInputDiagnosticSnapshot {
    pub timestamp: u64,
    pub source: &'static str,
    pub current_status: String,
    pub previous_status: String,
    pub battery: BatteryReading,
    pub battery_status: &'static str,
    pub remaining_capacity: f32,
    pub full_charge_capacity: f32,
    pub charge_rate: f32,
}

pub struct GameInputDiagnosticStream {
    _watcher: raw::CallbackWatcher,
    receiver: Receiver<raw::GameInputEvent>,
}

impl GameInputDiagnosticStream {
    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GameInputDiagnosticSnapshot, RecvTimeoutError> {
        self.receiver
            .recv_timeout(timeout)
            .map(GameInputDiagnosticSnapshot::from_event)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GameInputBackend;

impl GameInputBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnostic_snapshots(&self) -> AppResult<Vec<GameInputDiagnosticSnapshot>> {
        Ok(raw::enumerate_gamepad_snapshots()?
            .into_iter()
            .map(|snapshot| GameInputDiagnosticSnapshot::from_snapshot("device", snapshot))
            .collect())
    }

    pub fn start_diagnostic_event_stream(&self) -> AppResult<GameInputDiagnosticStream> {
        let (watcher, receiver) = raw::start_callback_watcher()?;

        Ok(GameInputDiagnosticStream {
            _watcher: watcher,
            receiver,
        })
    }

    fn controller_from_snapshot(snapshot: raw::GameInputDeviceSnapshot) -> Controller {
        Controller::new(
            snapshot.id,
            snapshot.name,
            ControllerSource::GameInput,
            snapshot.battery,
        )
    }
}

impl ControllerEventInput for GameInputBackend {
    fn start_event_stream(&self) -> AppResult<BackendEventStream> {
        let (watcher, receiver) = raw::start_callback_watcher()?;
        Ok(BackendEventStream::gameinput(watcher, receiver))
    }

    fn controller_from_event(&self, event: BackendEvent) -> (Controller, bool) {
        match event {
            BackendEvent::GameInput(event) => {
                let snapshot = event.into_snapshot();
                let is_connected = snapshot.is_connected();

                (Self::controller_from_snapshot(snapshot), is_connected)
            }
        }
    }
}

impl ControllerInput for GameInputBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(raw::enumerate_gamepad_snapshots()?
            .into_iter()
            .filter(|snapshot| snapshot.is_connected())
            .map(Self::controller_from_snapshot)
            .collect())
    }
}

impl ControllerRumbler for GameInputBackend {
    fn rumble(
        &self,
        _target: RumbleTarget,
        steps: &[RumbleStep],
    ) -> AppResult<Option<RumbleBackend>> {
        if raw::play_rumble_on_single_gamepad(steps)? {
            Ok(Some(RumbleBackend::GameInput))
        } else {
            Ok(None)
        }
    }
}

impl GameInputDiagnosticSnapshot {
    fn from_event(event: raw::GameInputEvent) -> Self {
        let source = event.source_label();
        Self::from_snapshot(source, event.into_snapshot())
    }

    fn from_snapshot(source: &'static str, snapshot: raw::GameInputDeviceSnapshot) -> Self {
        Self {
            timestamp: snapshot.timestamp,
            source,
            current_status: snapshot.current_status_description(),
            previous_status: snapshot.previous_status_description(),
            battery: snapshot.battery,
            battery_status: snapshot.battery_status_description(),
            remaining_capacity: snapshot.raw_battery.remaining_capacity,
            full_charge_capacity: snapshot.raw_battery.full_charge_capacity,
            charge_rate: snapshot.raw_battery.charge_rate,
        }
    }
}
