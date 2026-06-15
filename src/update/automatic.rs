use std::{
    path::Path,
    process::Command,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, SystemTime},
};

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

use super::{check, state::UpdateCheckState};
use crate::{
    AppResult,
    config::UpdatesConfig,
    notifier::{Notification, NotificationUrgency, Notifier, ToastNotifier},
    startup::StartupInstaller,
};

const STATE_FILE_NAME: &str = "update-state.toml";
const CHECK_LOOP_GRANULARITY: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct AutomaticUpdateHandle {
    config: Option<Arc<RwLock<UpdatesConfig>>>,
}

impl AutomaticUpdateHandle {
    fn disabled() -> Self {
        Self { config: None }
    }

    pub fn update_config(&self, config: UpdatesConfig) -> AppResult<()> {
        if let Some(current) = &self.config {
            *current
                .write()
                .map_err(|_| "automatic update config lock is poisoned")? = config;
        }

        Ok(())
    }
}

pub fn start_background_checks(
    config: UpdatesConfig,
    notifier: ToastNotifier,
) -> AppResult<AutomaticUpdateHandle> {
    let installer = StartupInstaller::new()?;
    let installed_exe = installer.installed_exe().to_path_buf();
    if !installed_exe.exists() {
        return Ok(AutomaticUpdateHandle::disabled());
    }

    let state_path = installed_exe.with_file_name(STATE_FILE_NAME);
    let config = Arc::new(RwLock::new(config));
    let thread_config = Arc::clone(&config);

    thread::spawn(move || {
        let _ = init_com_for_thread();

        loop {
            let config = match thread_config.read() {
                Ok(config) => config.clone(),
                Err(_) => {
                    eprintln!("automatic update config lock is poisoned");
                    break;
                }
            };

            if config.check_automatically
                && let Err(error) = run_due_check(&config, &notifier, &installed_exe, &state_path)
            {
                eprintln!("automatic update check failed: {error}");
            }

            thread::sleep(config.check_interval().min(CHECK_LOOP_GRANULARITY));
        }
    });

    Ok(AutomaticUpdateHandle {
        config: Some(config),
    })
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

fn notify_update_available(notifier: &ToastNotifier, latest_version: &str) {
    let notification = Notification::new(
        "xbattery Update Available",
        format!("Version {latest_version} is available. Run xbattery.exe update to install it."),
    );

    let _ = notifier.notify(&notification);
}

fn notify_auto_update_started(notifier: &ToastNotifier, latest_version: &str) {
    let notification = Notification::with_urgency(
        "xbattery Update Started",
        format!("Version {latest_version} is available. xbattery will restart after updating."),
        NotificationUrgency::High,
    );

    let _ = notifier.notify(&notification);
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

fn init_com_for_thread() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}
