use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::raw::{CallbackWatcher, GameInputEvent};

pub(crate) struct GameInputEventStream {
    _watcher: CallbackWatcher,
    receiver: Receiver<GameInputEvent>,
}

impl GameInputEventStream {
    pub(super) fn new(watcher: CallbackWatcher, receiver: Receiver<GameInputEvent>) -> Self {
        Self {
            _watcher: watcher,
            receiver,
        }
    }

    pub(crate) fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GameInputEvent, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}
