use std::{env, fs, path::Path, process::Command, time::Duration};

use super::{
    InstallReport, StartupStatus, UninstallReport, paths::StartupPaths, process::ProcessHandle,
    scheduled_task::ScheduledTask,
};
use crate::{
    AppResult,
    ipc::{BACKGROUND_INSTANCE_MUTEX_NAME, BACKGROUND_INSTANCE_STOP_EVENT_NAME, request_stop},
};

const DEFAULT_CONFIG: &str = include_str!("../../xbattery.toml");
const EXISTING_MONITOR_STOP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct StartupInstaller {
    paths: StartupPaths,
    task: ScheduledTask,
}

impl StartupInstaller {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            paths: StartupPaths::current_user()?,
            task: ScheduledTask::xbattery(),
        })
    }

    pub fn install(&self, start_now: bool, overwrite: bool) -> AppResult<InstallReport> {
        let status = self.status();
        if status.has_install_state() && !overwrite {
            return Err(format!(
                "xbattery already appears to be installed.\n\n{}\n\nRun `xbattery install --force` to overwrite the startup task and executable. Existing config is preserved.",
                status.summary()
            )
            .into());
        }

        if overwrite && status.installed_exe_exists {
            self.stop_existing_install()?;
        }

        fs::create_dir_all(&self.paths.install_dir)?;
        self.copy_exe()?;
        self.ensure_config()?;
        self.task.create(&self.paths.installed_exe)?;

        let started_monitor = if start_now {
            self.start_monitor()?;
            true
        } else {
            false
        };

        Ok(InstallReport {
            install_dir: self.paths.install_dir.clone(),
            installed_exe: self.paths.installed_exe.clone(),
            installed_config: self.paths.installed_config.clone(),
            started_monitor,
        })
    }

    pub fn uninstall(&self) -> AppResult<UninstallReport> {
        let task_removed = self.task.delete()?;

        Ok(UninstallReport {
            task_removed,
            install_dir: self.paths.install_dir.clone(),
        })
    }

    pub fn status(&self) -> StartupStatus {
        StartupStatus {
            task_exists: self.task.exists(),
            installed_exe_exists: self.paths.installed_exe.exists(),
            installed_config_exists: self.paths.installed_config.exists(),
            install_dir: self.paths.install_dir.clone(),
        }
    }

    pub fn installed_exe(&self) -> &Path {
        &self.paths.installed_exe
    }

    pub fn start_monitor(&self) -> AppResult<()> {
        let mut command = Command::new(&self.paths.installed_exe);
        command.arg("monitor");

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        command.spawn()?;
        Ok(())
    }

    fn stop_existing_install(&self) -> AppResult<()> {
        let _ = request_stop(
            BACKGROUND_INSTANCE_MUTEX_NAME,
            BACKGROUND_INSTANCE_STOP_EVENT_NAME,
            EXISTING_MONITOR_STOP_TIMEOUT,
        )?;
        for process in ProcessHandle::for_exe_path(&self.paths.installed_exe)? {
            process.terminate()?;
        }
        Ok(())
    }

    fn copy_exe(&self) -> AppResult<()> {
        let current_exe = env::current_exe()?;

        if StartupPaths::same_path(&current_exe, &self.paths.installed_exe) {
            return Ok(());
        }

        fs::copy(&current_exe, &self.paths.installed_exe)?;
        Ok(())
    }

    fn ensure_config(&self) -> AppResult<()> {
        if self.paths.installed_config.exists() {
            return Ok(());
        }

        if let Some(source_config) = StartupPaths::source_config_path()? {
            fs::copy(source_config, &self.paths.installed_config)?;
        } else {
            fs::write(&self.paths.installed_config, DEFAULT_CONFIG)?;
        }

        Ok(())
    }
}
