#![warn(unreachable_pub)]

mod app_config;
mod battery_config;
mod issue;
mod load;
mod monitor_config;
mod notifications_config;
mod updates;
mod validation;
mod watcher;

pub(crate) use app_config::{AppConfig, LoadedAppConfig};
pub(crate) use battery_config::BatteryConfig;
pub(crate) use issue::ConfigIssue;
pub(crate) use monitor_config::MonitorConfig;
pub(crate) use notifications_config::NotificationsConfig;
pub(crate) use updates::UpdatesConfig;
pub(crate) use watcher::ConfigWatchEvent;
pub(crate) use watcher::ConfigWatchEvents;

#[cfg(test)]
mod tests;
