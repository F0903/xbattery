mod backend;
#[cfg(debug_assertions)]
mod diagnostic_report;
mod native;
mod snapshot;

#[cfg(debug_assertions)]
pub(crate) use backend::diagnostic_reports;
pub(crate) use backend::poll_controllers;
#[cfg(debug_assertions)]
pub(crate) use diagnostic_report::XInputDiagnosticReport;
