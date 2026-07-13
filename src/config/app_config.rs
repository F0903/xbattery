use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{BatteryConfig, MonitorConfig, NotificationsConfig, UpdatesConfig, load, validation};
use crate::{
    AppResult,
    controller::{
        battery::BatteryWarningPolicy, event::ControllerNotificationPolicy,
        service::ControllerServiceConfig,
    },
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub monitor: MonitorConfig,
    pub battery: BatteryConfig,
    pub notifications: NotificationsConfig,
    pub updates: UpdatesConfig,
}

#[derive(Clone, Debug)]
pub struct LoadedAppConfig {
    pub config: AppConfig,
    pub path: Option<PathBuf>,
}

impl LoadedAppConfig {
    pub(super) fn new(config: AppConfig, path: Option<PathBuf>) -> Self {
        Self { config, path }
    }
}

impl AppConfig {
    pub fn load() -> AppResult<Self> {
        load::load()
    }

    pub fn load_with_source() -> AppResult<LoadedAppConfig> {
        load::load_with_source()
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> AppResult<Self> {
        load::load_from_path(path)
    }

    pub(super) fn resolve_relative_paths(&mut self, base_dir: Option<&Path>) {
        self.battery.resolve_relative_paths(base_dir);
    }

    pub fn controller_service_config(&self) -> AppResult<ControllerServiceConfig> {
        Ok(ControllerServiceConfig::new(
            self.monitor.poll_interval(),
            self.monitor.control_wait_slice(),
            BatteryWarningPolicy::new(self.battery.warning_levels()?),
            ControllerNotificationPolicy::new(
                self.notifications.notify_connected,
                self.notifications.notify_disconnected,
            ),
        ))
    }

    pub(super) fn validate(&self) -> AppResult<()> {
        validation::validate(self)
    }
}
