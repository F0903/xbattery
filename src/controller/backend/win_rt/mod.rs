mod gamepad_report;
mod raw_controller_report;
mod windows_gaming_input;

pub(crate) use gamepad_report::GamepadReport;
pub(crate) use raw_controller_report::RawControllerReport;
pub(crate) use windows_gaming_input::{gamepad_reports, raw_controller_reports};
