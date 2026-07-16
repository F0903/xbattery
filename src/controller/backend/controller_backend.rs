use std::time::Duration;

use crate::{AppResult, controller::Controller};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControllerStreamStatus {
    Changed,
    TimedOut,
    Disconnected,
}

pub(crate) trait ControllerEventStream {
    fn wait_for_change(&self, timeout: Duration) -> ControllerStreamStatus;
}

pub(crate) trait ControllerBackend {
    type EventStream: ControllerEventStream;

    fn start_event_stream(&self) -> AppResult<Self::EventStream>;
    fn poll_controllers(&self) -> AppResult<Vec<Controller>>;
}
