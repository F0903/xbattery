use std::{sync::mpsc::RecvTimeoutError, time::Duration};

use crate::{AppResult, controller::Controller};

use super::{ControllerBackend, ControllerEventStream, ControllerStreamStatus, gameinput, xinput};

/// Composes the Windows controller APIs behind one domain-facing backend.
///
/// GameInput supplies topology wake events, while XInput remains the canonical
/// source for controller identity and battery snapshots.
pub(crate) struct WindowsControllerBackend;

impl ControllerBackend for WindowsControllerBackend {
    type EventStream = gameinput::GameInputEventStream;

    fn start_event_stream(&self) -> AppResult<Self::EventStream> {
        gameinput::start_event_stream()
    }

    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        xinput::poll_controllers()
    }
}

impl ControllerEventStream for gameinput::GameInputEventStream {
    fn wait_for_change(&self, timeout: Duration) -> ControllerStreamStatus {
        match self.recv_timeout(timeout) {
            Ok(_) => ControllerStreamStatus::Changed,
            Err(RecvTimeoutError::Timeout) => ControllerStreamStatus::TimedOut,
            Err(RecvTimeoutError::Disconnected) => ControllerStreamStatus::Disconnected,
        }
    }
}
