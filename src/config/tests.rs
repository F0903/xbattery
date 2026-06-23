use crate::controller::battery::BatteryLevel;

use super::AppConfig;

#[test]
fn parses_partial_config_with_defaults() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [notifications]
        app_id = "custom-app"
        "#,
    )
    .unwrap();

    assert_eq!(config.notifications.app_id, "custom-app");
    assert!(config.notifications.notify_connected);
    assert!(config.notifications.notify_disconnected);
    assert_eq!(config.monitor.poll_interval_seconds, 60);
    assert!(config.battery.levels.is_none());
    assert_eq!(config.battery.precise_warning_thresholds, None);
    assert_eq!(config.updates.repo_name, "xbattery");
    assert!(config.updates.check_automatically);
    assert_eq!(config.updates.check_interval_hours, 24);
    assert!(!config.updates.auto_install);
    assert!(config.updates.notify_available);
}

#[test]
fn parses_optional_connectivity_notifications() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [notifications]
        notify_connected = false
        notify_disconnected = false
        "#,
    )
    .unwrap();

    assert!(!config.notifications.notify_connected);
    assert!(!config.notifications.notify_disconnected);
}

#[test]
fn parses_battery_level_config() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.medium]
        threshold_percent = 55
        coarse_level = "medium"
        urgent = false

        [battery.levels.empty]
        threshold_percent = 15
        coarse_level = "empty"
        urgent = true
        "#,
    )
    .unwrap();

    let levels = config.battery.warning_levels(None);

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].name(), "empty");
    assert_eq!(levels[0].precise_threshold_percent(), Some(15));
    assert_eq!(levels[0].coarse_level(), Some(BatteryLevel::Empty));
    assert!(levels[0].urgent());
    assert_eq!(levels[1].name(), "medium");
    assert_eq!(levels[1].precise_threshold_percent(), Some(55));
    assert_eq!(levels[1].coarse_level(), Some(BatteryLevel::Medium));
    assert!(!levels[1].urgent());
}

#[test]
fn default_battery_levels_use_legacy_urgent_threshold_when_present() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [notifications]
        urgent_precise_threshold_percent = 25
        "#,
    )
    .unwrap();

    let levels = config
        .battery
        .warning_levels(config.notifications.urgent_precise_threshold_percent);

    let medium = levels
        .iter()
        .find(|level| level.name() == "medium")
        .unwrap();
    let full = levels.iter().find(|level| level.name() == "full").unwrap();
    let low = levels.iter().find(|level| level.name() == "low").unwrap();
    let empty = levels.iter().find(|level| level.name() == "empty").unwrap();

    assert_eq!(full.precise_threshold_percent(), Some(100));
    assert_eq!(full.coarse_level(), Some(BatteryLevel::Full));
    assert!(!full.notify());
    assert!(!medium.urgent());
    assert!(!low.urgent());
    assert!(empty.urgent());
}

#[test]
fn parses_legacy_precise_warning_thresholds() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery]
        precise_warning_thresholds = [60, 30, 15]
        "#,
    )
    .unwrap();

    let levels = config.battery.warning_levels(None);

    assert!(
        levels
            .iter()
            .any(|level| level.precise_threshold_percent() == Some(60))
    );
    assert!(
        levels
            .iter()
            .any(|level| level.precise_threshold_percent() == Some(30))
    );
    assert!(
        levels
            .iter()
            .any(|level| level.precise_threshold_percent() == Some(15))
    );
}

#[test]
fn rejects_battery_level_without_matcher() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        urgent = true
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(
        error
            .to_string()
            .contains("threshold_percent or coarse_level")
    );
}

#[test]
fn rejects_duplicate_battery_level_thresholds() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.first]
        threshold_percent = 25

        [battery.levels.second]
        threshold_percent = 25
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("configured more than once"));
}

#[test]
fn parses_update_config() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [updates]
        repo_owner = "owner"
        repo_name = "repo"
        asset_identifier = "asset"
        bin_path_in_archive = "bin/xbattery.exe"
        check_automatically = false
        check_interval_hours = 12
        auto_install = true
        notify_available = false
        "#,
    )
    .unwrap();

    assert_eq!(config.updates.repo_owner, "owner");
    assert_eq!(config.updates.repo_name, "repo");
    assert_eq!(config.updates.asset_identifier, "asset");
    assert_eq!(config.updates.bin_path_in_archive, "bin/xbattery.exe");
    assert!(!config.updates.check_automatically);
    assert_eq!(config.updates.check_interval_hours, 12);
    assert!(config.updates.auto_install);
    assert!(!config.updates.notify_available);
}

#[test]
fn rejects_empty_update_repo_owner() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [updates]
        repo_owner = ""
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("updates.repo_owner"));
}

#[test]
fn rejects_zero_update_check_interval() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [updates]
        check_interval_hours = 0
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("updates.check_interval_hours"));
}

#[test]
fn rejects_unknown_fields() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [monitor]
        unsupported = true
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsupported"));
}
