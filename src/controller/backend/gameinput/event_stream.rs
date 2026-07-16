use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::raw::{CallbackWatcher, GameInputDeviceSnapshot};

pub(crate) struct GameInputEventStream {
    _watcher: CallbackWatcher,
    receiver: Receiver<GameInputDeviceSnapshot>,
}

impl GameInputEventStream {
    pub(super) fn new(
        watcher: CallbackWatcher,
        receiver: Receiver<GameInputDeviceSnapshot>,
    ) -> Self {
        Self {
            _watcher: watcher,
            receiver,
        }
    }

    pub(crate) fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GameInputDeviceSnapshot, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}
