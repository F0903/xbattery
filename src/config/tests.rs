use crate::controller::rumble::BatteryWarningStage;

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
    assert!(!config.rumble.enabled);
    assert_eq!(config.monitor.poll_interval_seconds, 60);
    assert_eq!(config.battery.precise_warning_thresholds, vec![50, 25, 10]);
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
fn parses_rumble_config() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [rumble]
        enabled = true
        gap_millis = 75
        group_gap_millis = 150

        [rumble.jolts.quick]
        handle_strength_percent = 85
        trigger_strength_percent = 55
        handle_millis = 70
        trigger_millis = 60
        "#,
    )
    .unwrap();

    assert!(config.rumble.enabled);
    assert_eq!(config.rumble.gap_millis, 75);
    assert_eq!(config.rumble.group_gap_millis, 150);

    let runtime = config.rumble.controller_rumble_config().unwrap();
    assert_eq!(runtime.jolt_gap_duration.as_millis(), 75);
    assert_eq!(runtime.group_gap_duration.as_millis(), 150);
}

#[test]
fn resolves_named_rumble_patterns() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [rumble]
        gap_millis = 40
        group_gap_millis = 120

        [rumble.jolts.soft]
        handle_strength_percent = 60
        trigger_strength_percent = 30
        handle_millis = 20
        trigger_millis = 25

        [rumble.patterns.medium]
        groups = [["soft", "quick"]]
        "#,
    )
    .unwrap();

    let runtime = config.rumble.controller_rumble_config().unwrap();
    let medium = runtime.pattern_for_stage(BatteryWarningStage::Medium);

    assert_eq!(medium.groups.len(), 1);
    assert_eq!(medium.groups[0].len(), 2);
    assert_eq!(medium.groups[0][0].handle_strength_percent, 60);
    assert_eq!(medium.groups[0][0].trigger_strength_percent, 30);
    assert_eq!(medium.groups[0][0].handle_phase_duration.as_millis(), 20);
    assert_eq!(medium.groups[0][0].trigger_phase_duration.as_millis(), 25);
}

#[test]
fn rejects_unknown_rumble_pattern_jolts() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [rumble.patterns.medium]
        groups = [["missing"]]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("unknown jolt"));
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
