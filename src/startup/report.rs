use std::path::PathBuf;

use super::TASK_NAME;

#[derive(Clone, Debug)]
pub struct InstallReport {
    pub installed_exe: PathBuf,
    pub installed_config: PathBuf,
}

#[derive(Clone, Debug)]
pub struct UninstallReport {
    pub task_removed: bool,
    pub install_dir: PathBuf,
}

impl InstallReport {
    pub fn summary(&self) -> String {
        format!(
            "xbattery is installed.\n\nExecutable: {}\nConfig: {}\nStartup task: {}\nMonitor started: yes",
            self.installed_exe.display(),
            self.installed_config.display(),
            TASK_NAME,
        )
    }
}

impl UninstallReport {
    pub fn summary(&self) -> String {
        format!(
            "xbattery startup task removed: {}\n\nInstalled files were left in place:\n{}",
            if self.task_removed {
                "yes"
            } else {
                "already absent"
            },
            self.install_dir.display()
        )
    }
}
