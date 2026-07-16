mod backend;
#[cfg(debug_assertions)]
mod diagnostic_snapshot;
#[cfg(debug_assertions)]
mod diagnostic_stream;
mod event_stream;
mod raw;

pub(crate) use backend::start_event_stream;
#[cfg(debug_assertions)]
pub(crate) use backend::{diagnostic_snapshots, start_diagnostic_event_stream};
#[cfg(debug_assertions)]
pub(crate) use diagnostic_snapshot::GameInputDiagnosticSnapshot;
#[cfg(debug_assertions)]
pub(crate) use diagnostic_stream::GameInputDiagnosticStream;
pub(crate) use event_stream::GameInputEventStream;
