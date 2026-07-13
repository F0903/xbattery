use std::{fs, path::PathBuf, time::SystemTime};

use crate::{audio::AudioClip, controller::battery::BatteryLevel};

use super::AppConfig;

#[test]
fn bundled_config_is_valid() {
    let config = toml::from_str::<AppConfig>(include_str!("../../xbattery.toml")).unwrap();

    config.validate().unwrap();
    assert_eq!(config.battery.warning_levels().unwrap().len(), 4);
}

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

        generated_sound.roll = [
            { notes = "A5", at = 0.0, length = 0.1, gain = 0.3, adsr = [0.008, 0.04, 0.08, 0.028] },
            { wave = "triangle", notes = "A6", at = 0.02, length = 0.06, gain = 0.08, adsr = [0.008, 0.02, 0.0, 0.028] },
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

    let levels = config.battery.warning_levels().unwrap();

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].name(), "empty");
    assert_eq!(levels[0].precise_threshold_percent(), Some(15));
    assert_eq!(levels[0].coarse_level(), Some(BatteryLevel::Empty));
    assert!(matches!(levels[0].audio(), Some(AudioClip::WavBytes(_))));
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
    let levels = config.battery.warning_levels().unwrap();
    let low = levels.iter().find(|level| level.name() == "low").unwrap();

    assert_eq!(
        low.audio(),
        Some(&AudioClip::file(temp_dir.join("sounds/low.wav")))
    );

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn warning_levels_builds_generated_battery_level_audio() {
    let temp_dir = unique_temp_dir("xbattery-generated-config-test");
    fs::create_dir_all(&temp_dir).unwrap();

    let config_path = temp_dir.join("xbattery.toml");
    fs::write(
        &config_path,
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", at = 0.0, length = 0.18, gain = 0.20, adsr = [0.008, 0.07, 0.08, 0.028] },
            { wave = "triangle", notes = "C6", at = 0.01, length = 0.08, gain = 0.07, adsr = [0.008, 0.03, 0.0, 0.028] },
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
    let levels = config.battery.warning_levels().unwrap();
    let low = levels.iter().find(|level| level.name() == "low").unwrap();

    match low.audio().unwrap() {
        AudioClip::WavBytes(bytes) => assert!(bytes.starts_with(b"RIFF")),
        AudioClip::File(path) => panic!("expected generated audio, got {}", path.display()),
    }

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn warning_levels_builds_audio_from_note_segments() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.segments = [
            { kind = "tone", notes = "C5 E5 G5", duration_seconds = 0.1 },
            { kind = "silence", duration_seconds = 0.04 },
            { kind = "tone", notes = "C6+7.5c", duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap();

    config.validate().unwrap();
    let levels = config.battery.warning_levels().unwrap();

    assert!(matches!(levels[0].audio(), Some(AudioClip::WavBytes(_))));
}

#[test]
fn default_battery_levels_have_fixed_urgency() {
    let levels = AppConfig::default().battery.warning_levels().unwrap();

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
fn rejects_generated_tone_without_notes() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        [battery.levels.low.generated_sound]

        [[battery.levels.low.generated_sound.segments]]
        kind = "tone"
        duration_seconds = 0.1
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("notes"));
}

#[test]
fn rejects_roll_event_without_notes() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { wave = "triangle", length = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("notes"));
}

#[test]
fn rejects_invalid_generated_note() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5 H4", length = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();
    let message = error.to_string();

    assert!(message.contains("notes[1]"));
    assert!(message.contains("H4"));
}

#[test]
fn rejects_note_above_sample_rate_nyquist_limit() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.sample_rate = 8000
        generated_sound.roll = [
            { notes = "C8", length = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("Nyquist"));
}

#[test]
fn rejects_notes_on_silence_segment() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.segments = [
            { kind = "silence", notes = "C4", duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("not valid for silence"));
}

#[test]
fn rejects_removed_frequency_config() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "A4", length = 0.1, frequencies = [440.0] },
        ]
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("frequencies"));
}

#[test]
fn rejects_removed_layer_notation() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.layers = [
            { notes = ["C5"], duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("layers"));
}

#[test]
fn rejects_wrong_adsr_length() {
    let error = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", length = 0.1, adsr = [0.01, 0.02, 0.5] },
        ]
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("adsr"));
}

#[test]
fn rejects_generated_sound_with_roll_and_segments() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", length = 0.1 },
        ]
        generated_sound.segments = [
            { kind = "tone", notes = "C5", duration_seconds = 0.1 },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("both roll and segments"));
}

#[test]
fn rejects_generated_delay_with_soft_limiter_fields() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", length = 0.1 },
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
fn rejects_roll_sustain_above_one() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", length = 0.1, adsr = [0.008, 0.0, 1.1, 0.028] },
        ]
        "#,
    )
    .unwrap();

    let error = config.validate().unwrap_err();

    assert!(error.to_string().contains("adsr[2] (sustain)"));
}

#[test]
fn rejects_generated_low_pass_with_delay_fields() {
    let config = toml::from_str::<AppConfig>(
        r#"
        [battery.levels.low]
        threshold_percent = 40

        generated_sound.roll = [
            { notes = "C5", length = 0.1 },
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

        generated_sound.roll = [
            { notes = "C5", length = 0.1 },
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
