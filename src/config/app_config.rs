use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{
    BatteryConfig, ConfigIssue, ConfigWatchEvents, MonitorConfig, NotificationsConfig,
    UpdatesConfig, load, validation,
};
use crate::{
    AppResult,
    controller::{
        battery::BatteryWarningPolicy, event::ControllerNotificationPolicy,
        service::ControllerServiceConfig,
    },
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AppConfig {
    pub(crate) monitor: MonitorConfig,
    pub(crate) battery: BatteryConfig,
    pub(crate) notifications: NotificationsConfig,
    pub(crate) updates: UpdatesConfig,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedAppConfig {
    pub(crate) config: AppConfig,
    pub(crate) path: Option<PathBuf>,
    pub(crate) issue: Option<ConfigIssue>,
}

impl LoadedAppConfig {
    pub(super) fn new(
        config: AppConfig,
        path: Option<PathBuf>,
        issue: Option<ConfigIssue>,
    ) -> Self {
        Self {
            config,
            path,
            issue,
        }
    }
}

impl AppConfig {
    pub(crate) fn load() -> AppResult<Self> {
        Ok(load::load_with_source()?.config)
    }

    #[cfg(debug_assertions)]
    pub(crate) fn load_with_source() -> AppResult<LoadedAppConfig> {
        load::load_with_source()
    }

    pub(crate) fn load_for_monitor() -> AppResult<(LoadedAppConfig, Option<ConfigWatchEvents>)> {
        load::load_for_monitor()
    }

    pub(crate) fn load_from_path(path: impl AsRef<Path>) -> AppResult<Self> {
        load::load_from_path(path)
    }

    pub(super) fn resolve_relative_paths(&mut self, base_dir: Option<&Path>) {
        self.battery.resolve_relative_paths(base_dir);
    }

    pub(crate) fn controller_service_config(&self) -> AppResult<ControllerServiceConfig> {
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
