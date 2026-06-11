use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    AppResult,
    battery::BatteryWarningPolicy,
    controller::{
        event::ControllerNotificationPolicy, rumble::ControllerRumbleConfig,
        service::ControllerServiceConfig,
    },
    toast::ToastConfig,
};

const DEFAULT_CONFIG_FILE_NAME: &str = "xbattery.toml";
const CONFIG_ENV_VAR: &str = "XBATTERY_CONFIG";

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub monitor: MonitorConfig,
    pub battery: BatteryConfig,
    pub notifications: NotificationsConfig,
    pub rumble: RumbleConfig,
}

impl AppConfig {
    pub fn load() -> AppResult<Self> {
        if let Some(path) = env::var_os(CONFIG_ENV_VAR).map(PathBuf::from) {
            return Self::load_from_path(path);
        }

        for path in default_config_paths()? {
            if path.exists() {
                return Self::load_from_path(path);
            }
        }

        Ok(Self::default())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
        let config = toml::from_str::<Self>(&content)
            .map_err(|error| format!("failed to parse {}: {}", path.display(), error))?;

        config.validate()?;
        Ok(config)
    }

    pub fn controller_service_config(&self) -> ControllerServiceConfig {
        ControllerServiceConfig::new(
            self.monitor.poll_interval(),
            self.monitor.control_wait_slice(),
            BatteryWarningPolicy::new(self.battery.precise_warning_thresholds.clone()),
            ControllerNotificationPolicy::new(self.notifications.urgent_precise_threshold_percent)
                .with_connectivity_notifications(
                    self.notifications.notify_connected,
                    self.notifications.notify_disconnected,
                ),
            self.rumble.controller_rumble_config(),
        )
    }

    pub fn toast_config(&self) -> ToastConfig {
        ToastConfig::new(self.notifications.app_id.clone())
    }

    fn validate(&self) -> AppResult<()> {
        if self.monitor.poll_interval_seconds == 0 {
            return Err("monitor.poll_interval_seconds must be greater than zero".into());
        }

        if self.monitor.control_wait_slice_millis == 0 {
            return Err("monitor.control_wait_slice_millis must be greater than zero".into());
        }

        if self.battery.precise_warning_thresholds.is_empty() {
            return Err("battery.precise_warning_thresholds must not be empty".into());
        }

        if self
            .battery
            .precise_warning_thresholds
            .iter()
            .any(|threshold| *threshold > 100)
        {
            return Err(
                "battery.precise_warning_thresholds values must be between 0 and 100".into(),
            );
        }

        if self.notifications.urgent_precise_threshold_percent > 100 {
            return Err(
                "notifications.urgent_precise_threshold_percent must be between 0 and 100".into(),
            );
        }

        if self.notifications.app_id.trim().is_empty() {
            return Err("notifications.app_id must not be empty".into());
        }

        if self.rumble.motor_strength_percent > 100 {
            return Err("rumble.motor_strength_percent must be between 0 and 100".into());
        }

        if self.rumble.pulse_millis == 0 {
            return Err("rumble.pulse_millis must be greater than zero".into());
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            monitor: MonitorConfig::default(),
            battery: BatteryConfig::default(),
            notifications: NotificationsConfig::default(),
            rumble: RumbleConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MonitorConfig {
    pub poll_interval_seconds: u64,
    pub control_wait_slice_millis: u64,
}

impl MonitorConfig {
    pub fn poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.poll_interval_seconds)
    }

    pub fn control_wait_slice(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.control_wait_slice_millis)
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 60,
            control_wait_slice_millis: 250,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryConfig {
    pub precise_warning_thresholds: Vec<u8>,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            precise_warning_thresholds: vec![50, 25, 10],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationsConfig {
    pub app_id: String,
    pub notify_connected: bool,
    pub notify_disconnected: bool,
    pub urgent_precise_threshold_percent: u8,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            app_id: "xbattery".to_string(),
            notify_connected: true,
            notify_disconnected: true,
            urgent_precise_threshold_percent: 10,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RumbleConfig {
    pub enabled: bool,
    pub motor_strength_percent: u8,
    pub pulse_millis: u64,
    pub gap_millis: u64,
}

impl RumbleConfig {
    pub fn controller_rumble_config(&self) -> ControllerRumbleConfig {
        ControllerRumbleConfig::new(
            self.enabled,
            self.motor_strength_percent,
            std::time::Duration::from_millis(self.pulse_millis),
            std::time::Duration::from_millis(self.gap_millis),
        )
    }
}

impl Default for RumbleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            motor_strength_percent: 35,
            pulse_millis: 120,
            gap_millis: 100,
        }
    }
}

fn default_config_paths() -> AppResult<Vec<PathBuf>> {
    let mut paths = vec![env::current_dir()?.join(DEFAULT_CONFIG_FILE_NAME)];

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_config = exe_dir.join(DEFAULT_CONFIG_FILE_NAME);
            if !paths.iter().any(|path| path == &exe_config) {
                paths.push(exe_config);
            }
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn parses_partial_config_with_defaults() {
        let config = toml::from_str::<AppConfig>(
            r#"
            [notifications]
            app_id = "custom-app"
            "#,
        )
        .unwrap();

        assert_eq!(config.notifications.app_id, "custom-app");
        assert!(config.notifications.notify_connected);
        assert!(config.notifications.notify_disconnected);
        assert!(!config.rumble.enabled);
        assert_eq!(config.monitor.poll_interval_seconds, 60);
        assert_eq!(config.battery.precise_warning_thresholds, vec![50, 25, 10]);
    }

    #[test]
    fn parses_optional_connectivity_notifications() {
        let config = toml::from_str::<AppConfig>(
            r#"
            [notifications]
            notify_connected = false
            notify_disconnected = false
            "#,
        )
        .unwrap();

        assert!(!config.notifications.notify_connected);
        assert!(!config.notifications.notify_disconnected);
    }

    #[test]
    fn parses_rumble_config() {
        let config = toml::from_str::<AppConfig>(
            r#"
            [rumble]
            enabled = true
            motor_strength_percent = 40
            pulse_millis = 150
            gap_millis = 75
            "#,
        )
        .unwrap();

        assert!(config.rumble.enabled);
        assert_eq!(config.rumble.motor_strength_percent, 40);
        assert_eq!(config.rumble.pulse_millis, 150);
        assert_eq!(config.rumble.gap_millis, 75);
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = toml::from_str::<AppConfig>(
            r#"
            [monitor]
            unsupported = true
            "#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unsupported"));
    }
}
