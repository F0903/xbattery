mod backend;
#[cfg(debug_assertions)]
mod diagnostic_report;
mod native;
mod snapshot;

pub use backend::XInputBackend;
#[cfg(debug_assertions)]
pub use diagnostic_report::XInputDiagnosticReport;
