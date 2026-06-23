use std::collections::HashSet;

use super::AppConfig;
use crate::AppResult;

pub(super) fn validate(config: &AppConfig) -> AppResult<()> {
    if config.monitor.poll_interval_seconds == 0 {
        return Err("monitor.poll_interval_seconds must be greater than zero".into());
    }

    if config.monitor.control_wait_slice_millis == 0 {
        return Err("monitor.control_wait_slice_millis must be greater than zero".into());
    }

    validate_battery_config(config)?;

    if config
        .notifications
        .urgent_precise_threshold_percent
        .is_some_and(|threshold| threshold > 100)
    {
        return Err(
            "notifications.urgent_precise_threshold_percent must be between 0 and 100".into(),
        );
    }

    if config.notifications.app_id.trim().is_empty() {
        return Err("notifications.app_id must not be empty".into());
    }

    if config.updates.repo_owner.trim().is_empty() {
        return Err("updates.repo_owner must not be empty".into());
    }

    if config.updates.repo_name.trim().is_empty() {
        return Err("updates.repo_name must not be empty".into());
    }

    if config.updates.asset_identifier.trim().is_empty() {
        return Err("updates.asset_identifier must not be empty".into());
    }

    if config.updates.bin_path_in_archive.trim().is_empty() {
        return Err("updates.bin_path_in_archive must not be empty".into());
    }

    if config.updates.check_interval_hours == 0 {
        return Err("updates.check_interval_hours must be greater than zero".into());
    }

    Ok(())
}

fn validate_battery_config(config: &AppConfig) -> AppResult<()> {
    if let Some(thresholds) = &config.battery.precise_warning_thresholds {
        if thresholds.is_empty() {
            return Err("battery.precise_warning_thresholds must not be empty".into());
        }

        if thresholds.iter().any(|threshold| *threshold > 100) {
            return Err(
                "battery.precise_warning_thresholds values must be between 0 and 100".into(),
            );
        }
    }

    let Some(levels) = &config.battery.levels else {
        return Ok(());
    };

    if levels.is_empty() {
        return Err("battery.levels must not be empty".into());
    }

    let mut threshold_percents = HashSet::new();
    let mut coarse_levels = HashSet::new();

    for (name, level) in levels {
        let level_path = format!("battery.levels.{name}");

        if name.trim().is_empty() {
            return Err("battery level names must not be empty".into());
        }

        if level.threshold_percent.is_none() && level.coarse_level.is_none() {
            return Err(
                format!("{level_path} must define threshold_percent or coarse_level").into(),
            );
        }

        if level
            .threshold_percent
            .is_some_and(|threshold| threshold > 100)
        {
            return Err(format!("{level_path}.threshold_percent must be between 0 and 100").into());
        }

        if let Some(threshold) = level.threshold_percent
            && !threshold_percents.insert(threshold)
        {
            return Err(format!(
                "battery warning threshold {threshold}% is configured more than once"
            )
            .into());
        }

        if let Some(coarse) = level.coarse_level
            && !coarse_levels.insert(coarse)
        {
            return Err(
                format!("battery coarse level {coarse} is configured more than once").into(),
            );
        }
    }

    Ok(())
}
