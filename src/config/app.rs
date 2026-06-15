use std::path::PathBuf;

use serde::Deserialize;

use super::{
    BatteryConfig, MonitorConfig, NotificationsConfig, RumbleConfig, UpdatesConfig, loader,
    validation,
};
use crate::{
    AppResult,
    battery::BatteryWarningPolicy,
    controller::{event::ControllerNotificationPolicy, service::ControllerServiceConfig},
    toast::ToastConfig,
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub monitor: MonitorConfig,
    pub battery: BatteryConfig,
    pub notifications: NotificationsConfig,
    pub rumble: RumbleConfig,
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
        loader::load()
    }

    pub fn load_with_source() -> AppResult<LoadedAppConfig> {
        loader::load_with_source()
    }

    pub fn load_from_path(path: impl AsRef<std::path::Path>) -> AppResult<Self> {
        loader::load_from_path(path)
    }

    pub fn controller_service_config(&self) -> AppResult<ControllerServiceConfig> {
        Ok(ControllerServiceConfig::new(
            self.monitor.poll_interval(),
            self.monitor.control_wait_slice(),
            BatteryWarningPolicy::new(self.battery.precise_warning_thresholds.clone()),
            ControllerNotificationPolicy::new(self.notifications.urgent_precise_threshold_percent)
                .with_connectivity_notifications(
                    self.notifications.notify_connected,
                    self.notifications.notify_disconnected,
                ),
            self.rumble.controller_rumble_config()?,
        ))
    }

    pub fn toast_config(&self) -> ToastConfig {
        ToastConfig::new(self.notifications.app_id.clone())
    }

    pub(super) fn validate(&self) -> AppResult<()> {
        validation::validate(self)
    }
}
