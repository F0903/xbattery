use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct UpdatesConfig {
    pub(crate) repo_owner: String,
    pub(crate) repo_name: String,
    pub(crate) asset_identifier: String,
    pub(crate) bin_path_in_archive: String,
    pub(crate) check_automatically: bool,
    pub(crate) check_interval_hours: u64,
    pub(crate) auto_install: bool,
    pub(crate) notify_available: bool,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            repo_owner: "F0903".to_string(),
            repo_name: "xbattery".to_string(),
            asset_identifier: "xbattery".to_string(),
            bin_path_in_archive: "xbattery.exe".to_string(),
            check_automatically: true,
            check_interval_hours: 24,
            auto_install: false,
            notify_available: true,
        }
    }
}

impl UpdatesConfig {
    pub(crate) fn check_interval(&self) -> Duration {
        Duration::from_secs(self.check_interval_hours.saturating_mul(60 * 60))
    }
}
