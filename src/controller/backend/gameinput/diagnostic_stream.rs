use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::{GameInputDiagnosticSnapshot, raw};

pub struct GameInputDiagnosticStream {
    _watcher: raw::CallbackWatcher,
    receiver: Receiver<raw::GameInputEvent>,
}

impl GameInputDiagnosticStream {
    pub(super) fn new(
        watcher: raw::CallbackWatcher,
        receiver: Receiver<raw::GameInputEvent>,
    ) -> Self {
        Self {
            _watcher: watcher,
            receiver,
        }
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GameInputDiagnosticSnapshot, RecvTimeoutError> {
        self.receiver
            .recv_timeout(timeout)
            .map(GameInputDiagnosticSnapshot::from_event)
    }
}
