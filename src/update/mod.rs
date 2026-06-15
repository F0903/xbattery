mod automatic;
mod report;
mod self_update;
mod state;

pub use automatic::{AutomaticUpdateHandle, start_background_checks};
pub use report::{CheckUpdateReport, UpdateReport};
pub use self_update::{check, update};
