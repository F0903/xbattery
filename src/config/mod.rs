mod app_config;
mod battery_config;
mod load;
mod monitor_config;
mod notifications_config;
mod rumble;
mod updates;
mod validation;
mod watcher;

pub use app_config::{AppConfig, LoadedAppConfig};
pub use battery_config::BatteryConfig;
pub use monitor_config::MonitorConfig;
pub use notifications_config::NotificationsConfig;
pub use rumble::RumbleConfig;
pub use updates::UpdatesConfig;
pub use watcher::watch as watch_config;

#[cfg(test)]
mod tests;
