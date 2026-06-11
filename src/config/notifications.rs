use serde::Deserialize;

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
