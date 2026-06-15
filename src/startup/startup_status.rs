use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct StartupStatus {
    pub task_exists: bool,
    pub installed_exe_exists: bool,
    pub installed_config_exists: bool,
    pub install_dir: PathBuf,
}

impl StartupStatus {
    pub fn has_install_state(&self) -> bool {
        self.task_exists || self.installed_exe_exists || self.installed_config_exists
    }

    pub fn summary(&self) -> String {
        format!(
            "Install dir: {}\nStartup task: {}\nExecutable: {}\nConfig: {}",
            self.install_dir.display(),
            if self.task_exists {
                "present"
            } else {
                "missing"
            },
            if self.installed_exe_exists {
                "present"
            } else {
                "missing"
            },
            if self.installed_config_exists {
                "present"
            } else {
                "missing"
            }
        )
    }
}
