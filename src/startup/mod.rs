mod error;
mod installer;
mod paths;
mod process;
mod report;
mod scheduled_task;
mod startup_status;

pub use error::{StartupAccessDenied, is_startup_access_denied};
pub use installer::StartupInstaller;
pub use report::{InstallReport, UninstallReport};
pub use startup_status::StartupStatus;

const TASK_NAME: &str = "xbattery";
