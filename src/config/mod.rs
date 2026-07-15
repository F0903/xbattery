mod app_config;
mod battery_config;
mod issue;
mod load;
mod monitor_config;
mod notifications_config;
mod updates;
mod validation;
mod watcher;

pub use app_config::{AppConfig, LoadedAppConfig};
pub use battery_config::BatteryConfig;
pub use issue::ConfigIssue;
pub use monitor_config::MonitorConfig;
pub use notifications_config::NotificationsConfig;
pub use updates::UpdatesConfig;
pub use watcher::ConfigWatchEvent;
pub(crate) use watcher::ConfigWatchEvents;

#[cfg(test)]
mod tests;
