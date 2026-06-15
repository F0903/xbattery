mod automatic;
mod report;
mod state;
mod updater;

pub use automatic::{AutomaticUpdateHandle, start_background_checks};
pub use report::{CheckUpdateReport, UpdateReport};
pub use updater::{check, update};
