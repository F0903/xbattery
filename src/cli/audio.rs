use std::{thread, time::Duration};

use crate::{AppResult, audio, config::AppConfig, controller::battery::BatteryWarningLevel};

const SOUND_TEST_PAUSE: Duration = Duration::from_millis(150);

pub(super) fn test_config() -> AppResult<()> {
    let loaded = AppConfig::load_with_source()?;
    let mut sounds = loaded
        .config
        .battery
        .warning_levels()?
        .into_iter()
        .filter(|level| level.audio().is_some())
        .collect::<Vec<_>>();

    sounds.sort_by(|left, right| {
        level_sort_percent(right)
            .cmp(&level_sort_percent(left))
            .then_with(|| left.name().cmp(right.name()))
    });

    match &loaded.path {
        Some(path) => println!("Loaded config from {}", path.display()),
        None => println!("No config file found; using built-in defaults."),
    }

    if sounds.is_empty() {
        println!("No battery level audio is configured.");
        return Ok(());
    }

    println!("Playing configured battery level audio:");
    for (index, level) in sounds.iter().enumerate() {
        let audio_clip = level.audio().expect("filtered levels with audio");

        match level_sort_percent(level) {
            Some(percent) => println!("  {} (~{percent}%): {}", level.name(), audio_clip),
            None => println!("  {}: {audio_clip}", level.name()),
        }

        audio::play_blocking(audio_clip)?;

        if index + 1 < sounds.len() {
            thread::sleep(SOUND_TEST_PAUSE);
        }
    }

    Ok(())
}

fn level_sort_percent(level: &BatteryWarningLevel) -> Option<u8> {
    level.precise_threshold_percent().or_else(|| {
        level
            .coarse_level()
            .and_then(|level| level.estimated_percent())
    })
}
