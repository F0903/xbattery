#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopResult {
    NotRunning,
    Stopped,
    TimedOut,
}
