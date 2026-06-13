mod report;

use std::{path::Path, time::Duration};

use self_update::{
    backends::github::Update,
    update::{Release, ReleaseUpdate, UpdateStatus},
};

pub use report::{CheckUpdateReport, UpdateReport};

use crate::{
    AppResult,
    config::UpdatesConfig,
    monitor_control::{MonitorStopResult, stop_monitor},
    startup::StartupInstaller,
};

const BIN_NAME: &str = "xbattery";
const MONITOR_STOP_TIMEOUT: Duration = Duration::from_secs(10);

pub fn check(config: &UpdatesConfig) -> AppResult<CheckUpdateReport> {
    let installer = StartupInstaller::new()?;
    let updater = updater(config, installer.installed_exe(), false)?;
    let release = available_release(updater.as_ref())?;

    Ok(match release {
        Some(release) => CheckUpdateReport::available(current_version(), release.version),
        None => CheckUpdateReport::up_to_date(current_version()),
    })
}

pub fn update(config: &UpdatesConfig, dry_run: bool) -> AppResult<UpdateReport> {
    let installer = StartupInstaller::new()?;
    let installed_exe = installer.installed_exe();
    if !installed_exe.exists() {
        return Err("xbattery is not installed. Run `xbattery install` before updating.".into());
    }

    let check_updater = updater(config, installed_exe, false)?;
    let Some(release) = available_release(check_updater.as_ref())? else {
        return Ok(UpdateReport::up_to_date(current_version()));
    };

    if dry_run {
        return Ok(UpdateReport::dry_run(current_version(), release.version));
    }

    let stop_result = stop_monitor(MONITOR_STOP_TIMEOUT)?;
    if stop_result == MonitorStopResult::TimedOut {
        return Err("timed out waiting for xbattery monitor to stop".into());
    }

    let update_result = run_update(config, installed_exe);
    match update_result {
        Ok(report) => {
            if stop_result == MonitorStopResult::Stopped {
                installer.start_monitor()?;
            }

            Ok(report.with_monitor_stop_result(stop_result))
        }
        Err(error) => {
            if stop_result == MonitorStopResult::Stopped {
                let _ = installer.start_monitor();
            }

            Err(error)
        }
    }
}

fn run_update(config: &UpdatesConfig, installed_exe: &Path) -> AppResult<UpdateReport> {
    let updater = updater(config, installed_exe, true)?;
    let status = updater
        .update_extended()
        .map_err(|error| format!("update failed: {error}"))?;

    Ok(match status {
        UpdateStatus::UpToDate => UpdateReport::up_to_date(current_version()),
        UpdateStatus::Updated(release) => UpdateReport::updated(current_version(), release.version),
    })
}

fn available_release(updater: &dyn ReleaseUpdate) -> AppResult<Option<Release>> {
    let releases = updater
        .get_latest_releases(&updater.current_version())
        .map_err(|error| format!("failed to check GitHub releases: {error}"))?;

    let target = updater.target();
    let identifier = updater.identifier();

    Ok(releases
        .into_iter()
        .find(|release| release.asset_for(&target, identifier.as_deref()).is_some()))
}

fn updater(
    config: &UpdatesConfig,
    installed_exe: &Path,
    show_download_progress: bool,
) -> AppResult<Box<dyn ReleaseUpdate>> {
    let mut builder = Update::configure();
    builder
        .repo_owner(config.repo_owner.trim())
        .repo_name(config.repo_name.trim())
        .bin_name(BIN_NAME)
        .bin_install_path(installed_exe)
        .bin_path_in_archive(config.bin_path_in_archive.trim())
        .identifier(config.asset_identifier.trim())
        .current_version(current_version())
        .show_download_progress(show_download_progress)
        .show_output(show_download_progress)
        .no_confirm(true);

    builder
        .build()
        .map_err(|error| format!("failed to configure updater: {error}").into())
}

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
