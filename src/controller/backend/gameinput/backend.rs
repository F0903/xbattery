use crate::AppResult;

#[cfg(debug_assertions)]
use super::{GameInputDiagnosticSnapshot, GameInputDiagnosticStream};
use super::{GameInputEventStream, raw};

#[cfg(debug_assertions)]
pub(crate) fn diagnostic_snapshots() -> AppResult<Vec<GameInputDiagnosticSnapshot>> {
    Ok(raw::enumerate_gamepad_snapshots()?
        .into_iter()
        .map(GameInputDiagnosticSnapshot::from_snapshot)
        .collect())
}

#[cfg(debug_assertions)]
pub(crate) fn start_diagnostic_event_stream() -> AppResult<GameInputDiagnosticStream> {
    let (watcher, receiver) = raw::start_callback_watcher()?;

    Ok(GameInputDiagnosticStream::new(watcher, receiver))
}

pub(crate) fn start_event_stream() -> AppResult<GameInputEventStream> {
    let (watcher, receiver) = raw::start_callback_watcher()?;
    Ok(GameInputEventStream::new(watcher, receiver))
}
