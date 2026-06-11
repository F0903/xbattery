use super::AppConfig;
use crate::AppResult;

pub(super) fn validate(config: &AppConfig) -> AppResult<()> {
    if config.monitor.poll_interval_seconds == 0 {
        return Err("monitor.poll_interval_seconds must be greater than zero".into());
    }

    if config.monitor.control_wait_slice_millis == 0 {
        return Err("monitor.control_wait_slice_millis must be greater than zero".into());
    }

    if config.battery.precise_warning_thresholds.is_empty() {
        return Err("battery.precise_warning_thresholds must not be empty".into());
    }

    if config
        .battery
        .precise_warning_thresholds
        .iter()
        .any(|threshold| *threshold > 100)
    {
        return Err("battery.precise_warning_thresholds values must be between 0 and 100".into());
    }

    if config.notifications.urgent_precise_threshold_percent > 100 {
        return Err(
            "notifications.urgent_precise_threshold_percent must be between 0 and 100".into(),
        );
    }

    if config.notifications.app_id.trim().is_empty() {
        return Err("notifications.app_id must not be empty".into());
    }

    if config.rumble.gap_millis == 0 {
        return Err("rumble.gap_millis must be greater than zero".into());
    }

    if config.rumble.group_gap_millis == 0 {
        return Err("rumble.group_gap_millis must be greater than zero".into());
    }

    config.rumble.controller_rumble_config()?;

    Ok(())
}
