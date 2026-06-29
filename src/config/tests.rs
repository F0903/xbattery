use std::{fs, path::Path, path::PathBuf, time::SystemTime};

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

        generated_sound.layers = [
            { waveform = "sine", frequencies = [880.0], start_seconds = 0.0, duration_seconds = 0.1, volume = 0.3, decay_seconds = 0.04, sustain_level = 0.08 },
            { waveform = "triangle", frequencies = [1760.0], start_seconds = 0.02, duration_seconds = 0.06, volume = 0.08, decay_seconds = 0.02, sustain_level = 0.0 },
        ]
        generated_sound.effects = [
            { kind = "low_pass", cutoff_hz = 1400.0 },
            { kind = "delay", delay_seconds = 0.04, feedback = 0.15, mix = 0.1 },
            { kind = "reverb", room_seconds = 0.18, damping = 0.45, mix = 0.10 },
            { kind = "soft_limiter", drive = 1.2 },
        ]
        "#,
    )
    .unwrap();

    let levels = config.battery.warning_levels(None);

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].name(), "empty");
    assert_eq!(levels[0].precise_threshold_percent(), Some(15));
    assert_eq!(levels[0].coarse_level(), Some(BatteryLevel::Empty));
    assert_eq!(levels[0].sound_file(), Some(Path::new("sounds/empty.wav")));
    assert!(levels[0].urgent());
    assert_eq!(levels[1].name(), "medium");
    assert_eq!(levels[1].precise_threshold_percent(), Some(55));
    assert_eq!(levels[1].coarse_level(), Some(BatteryLevel::Medium));
    assert!(!levels[1].urgent());
}

#[test]
fn load_from_path_resolves_relative_battery_level_sound_files() {
    let temp_dir = unique_temp_dir("xbattery-config-test");
    fs::create_dir_all(&temp_dir).unwrap();

    let config_path = temp_dir.join("xbattery.toml");
    fs::write(
        &config_path,
        r#"
        [battery.levels.low]
        threshold_percent = 40
        sound_file = "sounds/low.wav"
        "#,
    )
    .unwrap();

    let config = AppConfig::load_from_path(&config_path).unwrap();
    let levels = config.battery.warning_levels(None);
    let low = levels.iter().find(|level| level.name() == "low").unwrap();

    assert_eq!(
        low.sound_file(),
        Some(temp_dir.join("sounds/low.wav").as_path())
    );

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn load_from_path_generates_relative_battery_level_sound_files() {
    let temp_dir = unique_temp_dir("xbattery-generated-config-test");
    fs::create_dir_all(&temp_dir).unwrap();

    let config_path = temp_dir.join("xbattery.toml");
    fs::write(
        &config_path,
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], start_seconds = 0.0, duration_seconds = 0.18, volume = 0.20, decay_seconds = 0.07, sustain_level = 0.08 },
            { waveform = "triangle", frequencies = [1046.5], start_seconds = 0.01, duration_seconds = 0.08, volume = 0.07, decay_seconds = 0.03, sustain_level = 0.0 },
        ]
        generated_sound.effects = [
            { kind = "low_pass", cutoff_hz = 1200.0 },
            { kind = "delay", delay_seconds = 0.05, feedback = 0.15, mix = 0.12 },
            { kind = "reverb", room_seconds = 0.20, damping = 0.45, mix = 0.10 },
            { kind = "soft_limiter", drive = 1.2 },
        ]
        "#,
    )
    .unwrap();

    let config = AppConfig::load_from_path(&config_path).unwrap();
    let levels = config.battery.warning_levels(None);
    let low = levels.iter().find(|level| level.name() == "low").unwrap();
    let generated_file = temp_dir.join("sounds/low.wav");

    assert_eq!(low.sound_file(), Some(generated_file.as_path()));
    assert!(fs::read(generated_file).unwrap().starts_with(b"RIFF"));

    fs::remove_dir_all(temp_dir).unwrap();
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
fn rejects_non_wav_battery_level_sound_file() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40
        sound_file = "low.mp3"
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains(".sound_file"));
}

#[test]
fn rejects_battery_level_with_sound_file_and_generated_sound() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40
        sound_file = "low.wav"

        [battery.levels.low.generated_sound]
        file = "generated-low.wav"
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(
        error
            .to_string()
            .contains("both sound_file and generated_sound")
    );
}

#[test]
fn rejects_generated_tone_without_frequencies() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        [battery.levels.low.generated_sound]
        file = "sounds/low.wav"

        [[battery.levels.low.generated_sound.segments]]
        kind = "tone"
        duration_seconds = 0.1
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("frequencies"));
}

#[test]
fn rejects_generated_layer_without_frequencies() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "triangle", duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("frequencies"));
}

#[test]
fn rejects_generated_sound_with_layers_and_segments() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], duration_seconds = 0.1 },
        ]
        generated_sound.segments = [
            { kind = "tone", frequencies = [523.25], duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("both layers and segments"));
}

#[test]
fn rejects_generated_delay_with_soft_limiter_fields() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], duration_seconds = 0.1 },
        ]
        generated_sound.effects = [
            { kind = "delay", delay_seconds = 0.05, drive = 1.2 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("drive"));
}

#[test]
fn rejects_generated_layer_sustain_above_one() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], duration_seconds = 0.1, sustain_level = 1.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("sustain_level"));
}

#[test]
fn rejects_generated_low_pass_with_delay_fields() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], duration_seconds = 0.1 },
        ]
        generated_sound.effects = [
            { kind = "low_pass", cutoff_hz = 1200.0, mix = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("mix"));
}

#[test]
fn rejects_generated_reverb_room_above_limit() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { waveform = "sine", frequencies = [523.25], duration_seconds = 0.1 },
        ]
        generated_sound.effects = [
            { kind = "reverb", room_seconds = 3.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("room_seconds"));
}

#[test]
fn rejects_removed_generated_sound_preset_config() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        [battery.levels.low.generated_sound]
        preset = "low"
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("preset"));
}

#[test]
fn rejects_removed_toast_sound_config() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40
        sound = "alarm"
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("sound"));
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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
