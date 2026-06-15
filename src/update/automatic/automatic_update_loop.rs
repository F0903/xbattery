use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, SystemTime},
};

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

use crate::{
    AppResult,
    config::UpdatesConfig,
    notifier::ToastNotifier,
    startup::StartupInstaller,
    update::{check, state::UpdateCheckState},
};

use super::{
    automatic_update_handle::AutomaticUpdateHandle,
    notify::{notify_auto_update_started, notify_update_available},
};

const STATE_FILE_NAME: &str = "update-state.toml";
const CHECK_LOOP_GRANULARITY: Duration = Duration::from_secs(60);

pub(super) struct AutomaticUpdateLoop {
    config: Arc<RwLock<UpdatesConfig>>,
    notifier: ToastNotifier,
    installed_exe: PathBuf,
    state_path: PathBuf,
}

impl AutomaticUpdateLoop {
    pub(super) fn start(
        config: UpdatesConfig,
        notifier: ToastNotifier,
    ) -> AppResult<AutomaticUpdateHandle> {
        let installer = StartupInstaller::new()?;
        let installed_exe = installer.installed_exe().to_path_buf();
        if !installed_exe.exists() {
            return Ok(AutomaticUpdateHandle::disabled());
        }

        let config = Arc::new(RwLock::new(config));
        let update_loop = Self {
            config: Arc::clone(&config),
            notifier,
            state_path: installed_exe.with_file_name(STATE_FILE_NAME),
            installed_exe,
        };
        update_loop.spawn();

        Ok(AutomaticUpdateHandle::enabled(config))
    }

    fn spawn(self) {
        thread::spawn(move || self.run());
    }

    fn run(self) {
        let _ = init_com_for_thread();

        loop {
            let config = match self.current_config() {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("{error}");
                    break;
                }
            };

            if config.check_automatically
                && let Err(error) = self.run_due_check(&config)
            {
                eprintln!("automatic update check failed: {error}");
            }

            thread::sleep(config.check_interval().min(CHECK_LOOP_GRANULARITY));
        }
    }

    fn current_config(&self) -> AppResult<UpdatesConfig> {
        Ok(self
            .config
            .read()
            .map_err(|_| "automatic update config lock is poisoned")?
            .clone())
    }

    fn run_due_check(&self, config: &UpdatesConfig) -> AppResult<()> {
        run_due_check(
            config,
            &self.notifier,
            &self.installed_exe,
            &self.state_path,
        )
    }
}

fn spawn_update_process(installed_exe: &Path) -> AppResult<()> {
    let mut command = Command::new(installed_exe);
    command.arg("update");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.spawn()?;
    Ok(())
}

fn run_due_check(
    config: &UpdatesConfig,
    notifier: &ToastNotifier,
    installed_exe: &Path,
    state_path: &Path,
) -> AppResult<()> {
    let mut state = UpdateCheckState::load(state_path);
    let now = SystemTime::now();
    if !state.is_due(now, config.check_interval()) {
        return Ok(());
    }

    state.mark_checked(now);
    state.save(state_path)?;

    let report = check(config)?;
    let Some(latest_version) = report.latest_version() else {
        return Ok(());
    };

    if config.auto_install {
        notify_auto_update_started(notifier, latest_version);
        spawn_update_process(installed_exe)?;
        return Ok(());
    }

    if config.notify_available && state.should_notify_for(latest_version) {
        notify_update_available(notifier, latest_version);
        state.mark_notified(latest_version);
        state.save(state_path)?;
    }

    Ok(())
}

fn init_com_for_thread() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}
