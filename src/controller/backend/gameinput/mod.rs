mod backend;
#[cfg(debug_assertions)]
mod diagnostic_snapshot;
#[cfg(debug_assertions)]
mod diagnostic_stream;
mod event_stream;
mod raw;

pub use backend::GameInputBackend;
#[cfg(debug_assertions)]
pub use diagnostic_snapshot::GameInputDiagnosticSnapshot;
#[cfg(debug_assertions)]
pub use diagnostic_stream::GameInputDiagnosticStream;
pub(crate) use event_stream::GameInputEventStream;
