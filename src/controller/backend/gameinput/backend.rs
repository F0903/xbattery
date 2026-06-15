use crate::{
    AppResult,
    controller::{
        Controller, ControllerSource,
        backend::{
            BackendEvent, BackendEventStream, BackendKind, EventBackend, InputBackend,
            RumbleBackend,
        },
        rumble::{RumbleStep, RumbleTarget},
    },
};

use super::{GameInputDiagnosticSnapshot, GameInputDiagnosticStream, raw};

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

        Ok(GameInputDiagnosticStream::new(watcher, receiver))
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

impl EventBackend for GameInputBackend {
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

impl InputBackend for GameInputBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(raw::enumerate_gamepad_snapshots()?
            .into_iter()
            .filter(|snapshot| snapshot.is_connected())
            .map(Self::controller_from_snapshot)
            .collect())
    }
}

impl RumbleBackend for GameInputBackend {
    fn rumble(
        &self,
        _target: RumbleTarget,
        steps: &[RumbleStep],
    ) -> AppResult<Option<BackendKind>> {
        if raw::play_rumble_on_single_gamepad(steps)? {
            Ok(Some(BackendKind::GameInput))
        } else {
            Ok(None)
        }
    }
}
