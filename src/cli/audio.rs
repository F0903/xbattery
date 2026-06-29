#[cfg(debug_assertions)]
use std::{path::PathBuf, thread, time::Duration};

use xbattery::{AppResult, config::AppConfig};

#[cfg(debug_assertions)]
use xbattery::{audio, controller::battery::BatteryWarningLevel};

#[cfg(debug_assertions)]
const SOUND_TEST_PAUSE: Duration = Duration::from_millis(150);

pub(super) fn generate_sounds() -> AppResult<()> {
    let loaded = AppConfig::load_with_source()?;
    let files = loaded.config.generated_sound_files();

    if files.is_empty() {
        println!("No generated sounds are configured.");
        return Ok(());
    }

    println!("Generated configured sounds:");
    for file in files {
        println!("  {}", file.display());
    }

    Ok(())
}

#[cfg(debug_assertions)]
pub(super) fn test_file(path: PathBuf) -> AppResult<()> {
    println!("Playing {}", path.display());
    audio::play_file_blocking(&path)
}

#[cfg(debug_assertions)]
pub(super) fn test_config() -> AppResult<()> {
    let loaded = AppConfig::load_with_source()?;
    let mut sounds = loaded
        .config
        .battery
        .warning_levels(loaded.config.notifications.urgent_precise_threshold_percent)
        .into_iter()
        .filter_map(|level| {
            let path = level.sound_file()?.to_path_buf();
            Some((level_sort_percent(&level), level.name().to_owned(), path))
        })
        .collect::<Vec<_>>();

    sounds.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    match &loaded.path {
        Some(path) => println!("Loaded config from {}", path.display()),
        None => println!("No config file found; using built-in defaults."),
    }

    if sounds.is_empty() {
        println!("No battery level sounds are configured.");
        return Ok(());
    }

    println!("Playing configured battery level sounds:");
    for (index, (percent, name, path)) in sounds.iter().enumerate() {
        match percent {
            Some(percent) => println!("  {name} (~{percent}%): {}", path.display()),
            None => println!("  {name}: {}", path.display()),
        }

        audio::play_file_blocking(path)?;

        if index + 1 < sounds.len() {
            thread::sleep(SOUND_TEST_PAUSE);
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn level_sort_percent(level: &BatteryWarningLevel) -> Option<u8> {
    level
        .precise_threshold_percent()
        .or_else(|| level.coarse_level().map(|level| level.estimated_percent()))
}
