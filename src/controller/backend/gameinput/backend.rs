use crate::{AppResult, controller::Controller};

use super::{
    GameInputDiagnosticSnapshot, GameInputDiagnosticStream, GameInputEvent, GameInputEventStream,
    raw,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct GameInputBackend;

impl GameInputBackend {
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

    pub(crate) fn start_event_stream(&self) -> AppResult<GameInputEventStream> {
        let (watcher, receiver) = raw::start_callback_watcher()?;
        Ok(GameInputEventStream::new(watcher, receiver))
    }

    pub(crate) fn controller_from_event(&self, event: GameInputEvent) -> (Controller, bool) {
        let snapshot = event.into_snapshot();
        let is_connected = snapshot.is_connected();

        (Self::controller_from_snapshot(snapshot), is_connected)
    }

    pub(crate) fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(raw::enumerate_gamepad_snapshots()?
            .into_iter()
            .filter(|snapshot| snapshot.is_connected())
            .map(Self::controller_from_snapshot)
            .collect())
    }

    fn controller_from_snapshot(snapshot: raw::GameInputDeviceSnapshot) -> Controller {
        Controller::new(snapshot.id, snapshot.battery)
    }
}
