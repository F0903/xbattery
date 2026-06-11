mod installer;
mod paths;
mod report;
mod scheduled_task;
mod status;

pub use installer::StartupInstaller;
pub use report::{InstallReport, UninstallReport};
pub use status::StartupStatus;

const TASK_NAME: &str = "xbattery";
const EXE_NAME: &str = "xbattery.exe";
const CONFIG_NAME: &str = "xbattery.toml";
