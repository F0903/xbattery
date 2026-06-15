mod app;
mod battery;
mod loader;
mod monitor;
mod notifications;
mod rumble;
mod updates;
mod validation;
mod watcher;

pub use app::{AppConfig, LoadedAppConfig};
pub use battery::BatteryConfig;
pub use monitor::MonitorConfig;
pub use notifications::NotificationsConfig;
pub use rumble::RumbleConfig;
pub use updates::UpdatesConfig;
pub use watcher::watch as watch_config;

#[cfg(test)]
mod tests;
