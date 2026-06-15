#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorStopResult {
    NotRunning,
    Stopped,
    TimedOut,
}
