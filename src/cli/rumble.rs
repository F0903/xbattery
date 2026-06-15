use std::{thread, time::Duration};

use xbattery::{
    AppResult, config::AppConfig, controller::battery::BatteryLevel, controller::rumble,
};

pub(super) fn test() -> AppResult<()> {
    let config = AppConfig::load()?;
    let backend = rumble::rumble_single_controller(config.rumble.controller_rumble_config()?, 3)?;

    println!(
        "Sent the critical battery rumble pattern with {}.",
        backend.description()
    );
    Ok(())
}

pub(super) fn test_thresholds() -> AppResult<()> {
    const BETWEEN_PATTERNS: Duration = Duration::from_millis(1500);
    let patterns = [
        (BatteryLevel::Medium, 1, "configured medium-stage pattern"),
        (BatteryLevel::Low, 2, "configured low-stage pattern"),
        (BatteryLevel::Empty, 3, "configured empty-stage pattern"),
    ];
    let config = AppConfig::load()?;
    let rumble_config = config.rumble.controller_rumble_config()?;

    println!("Testing battery threshold rumble patterns.");

    for (index, (level, warning_level, description)) in patterns.iter().enumerate() {
        println!(
            "  ~{}% / {}: {}",
            level.estimated_percent(),
            level,
            description
        );
        let backend = rumble::rumble_single_controller(rumble_config.clone(), *warning_level)?;
        println!("    backend: {}", backend.description());

        if index + 1 < patterns.len() {
            thread::sleep(BETWEEN_PATTERNS);
        }
    }

    Ok(())
}
