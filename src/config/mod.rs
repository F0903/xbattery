mod app;
mod battery;
mod loader;
mod monitor;
mod notifications;
mod rumble;
mod updates;
mod validation;

pub use app::AppConfig;
pub use battery::BatteryConfig;
pub use monitor::MonitorConfig;
pub use notifications::NotificationsConfig;
pub use rumble::RumbleConfig;
pub use updates::UpdatesConfig;

#[cfg(test)]
mod tests;
