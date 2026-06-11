mod app;
mod battery;
mod loader;
mod monitor;
mod notifications;
mod rumble;
mod validation;

pub use app::AppConfig;
pub use battery::BatteryConfig;
pub use monitor::MonitorConfig;
pub use notifications::NotificationsConfig;
pub use rumble::RumbleConfig;

#[cfg(test)]
mod tests;
