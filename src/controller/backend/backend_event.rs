use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::gameinput::{CallbackWatcher, GameInputEvent};

pub enum BackendEvent {
    GameInput(GameInputEvent),
}

pub struct BackendEventStream {
    _watcher: CallbackWatcher,
    receiver: Receiver<GameInputEvent>,
}

impl BackendEventStream {
    pub(super) fn gameinput(watcher: CallbackWatcher, receiver: Receiver<GameInputEvent>) -> Self {
        Self {
            _watcher: watcher,
            receiver,
        }
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<BackendEvent, RecvTimeoutError> {
        self.receiver
            .recv_timeout(timeout)
            .map(BackendEvent::GameInput)
    }
}
