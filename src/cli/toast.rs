use xbattery::{AppResult, config::AppConfig, notifier::NotificationUrgency, toast::Toast};

pub(super) fn test() -> AppResult<()> {
    let config = AppConfig::load()?;
    Toast::with_config(
        config.toast_config(),
        "xbattery",
        "Toast notifications are working.",
    )
    .send()
}

pub(super) fn test_high() -> AppResult<()> {
    let config = AppConfig::load()?;
    Toast::with_config_and_urgency(
        config.toast_config(),
        "xbattery",
        "High priority toast notifications are working.",
        NotificationUrgency::High,
    )
    .send()
}

pub(super) fn test_urgent() -> AppResult<()> {
    let config = AppConfig::load()?;
    Toast::with_config_and_urgency(
        config.toast_config(),
        "xbattery",
        "Urgent toast notifications are working.",
        NotificationUrgency::Urgent,
    )
    .send()
}
