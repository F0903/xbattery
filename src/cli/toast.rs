use std::{thread, time::Duration};

use xbattery::{
    AppResult,
    config::AppConfig,
    controller::{
        Controller, ControllerSource,
        backend::BackendKind,
        battery::{
            BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning,
            BatteryWarningLevel,
        },
        event::{ControllerEvent, ControllerNotificationPolicy},
    },
    notifier::{NotificationUrgency, Notifier, ToastNotifier},
    toast::Toast,
};

const PREVIEW_DELAY: Duration = Duration::from_millis(900);

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

pub(super) fn preview() -> AppResult<()> {
    let config = AppConfig::load()?;
    let policy = ControllerNotificationPolicy::new().with_connectivity_notifications(true, true);
    let notifier = ToastNotifier::new(config.toast_config());
    let warning_levels = config
        .battery
        .warning_levels(config.notifications.urgent_precise_threshold_percent);
    let previews = preview_events(&warning_levels);

    println!("Sending xbattery notification preview.");

    for (index, preview) in previews.iter().enumerate() {
        println!("  {}", preview.label);

        if let Some(notification) = preview.event.notification(&policy) {
            notifier.notify(&notification)?;
        }

        if index + 1 < previews.len() {
            thread::sleep(PREVIEW_DELAY);
        }
    }

    Ok(())
}

struct PreviewEvent {
    label: String,
    event: ControllerEvent,
}

fn preview_events(warning_levels: &[BatteryWarningLevel]) -> Vec<PreviewEvent> {
    let mut previews = vec![
        PreviewEvent {
            label: "connected".to_string(),
            event: ControllerEvent::Connected(sample_controller(
                BackendKind::XInput,
                BatteryReading::new(
                    BatteryKind::Alkaline,
                    BatteryCharge::Coarse(BatteryLevel::Full),
                ),
            )),
        },
        PreviewEvent {
            label: "disconnected".to_string(),
            event: ControllerEvent::Disconnected(sample_controller(
                BackendKind::XInput,
                BatteryReading::new(BatteryKind::Disconnected, BatteryCharge::Unknown),
            )),
        },
    ];

    let mut precise_levels = warning_levels
        .iter()
        .filter(|level| level.notify())
        .filter_map(|level| {
            level
                .precise_threshold_percent()
                .map(|threshold| (threshold, level.clone()))
        })
        .collect::<Vec<_>>();
    precise_levels.sort_by(|(left, _), (right, _)| right.cmp(left));

    for (threshold, level) in precise_levels {
        previews.push(precise_warning(threshold, level));
    }

    let mut coarse_levels = warning_levels
        .iter()
        .filter(|level| level.notify())
        .filter_map(|level| {
            level
                .coarse_level()
                .map(|coarse_level| (coarse_level, level.clone()))
        })
        .collect::<Vec<_>>();
    coarse_levels.sort_by(|(left, _), (right, _)| right.cmp(left));

    for (coarse_level, level) in coarse_levels {
        previews.push(coarse_warning(coarse_level, level));
    }

    previews
}

fn precise_warning(percent: u8, level: BatteryWarningLevel) -> PreviewEvent {
    PreviewEvent {
        label: warning_label(&level, format!("{percent}% battery warning")),
        event: ControllerEvent::BatteryWarning {
            current: sample_controller(
                BackendKind::GameInput,
                BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Precise(percent)),
            ),
            warning: BatteryWarning::precise(percent, level),
        },
    }
}

fn coarse_warning(coarse_level: BatteryLevel, level: BatteryWarningLevel) -> PreviewEvent {
    PreviewEvent {
        label: warning_label(&level, format!("{coarse_level} coarse battery warning")),
        event: ControllerEvent::BatteryWarning {
            current: sample_controller(
                BackendKind::XInput,
                BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(coarse_level)),
            ),
            warning: BatteryWarning::coarse(coarse_level, level),
        },
    }
}

fn warning_label(level: &BatteryWarningLevel, detail: String) -> String {
    let urgency = if level.urgent() {
        "urgent"
    } else {
        "high priority"
    };
    format!("{}: {detail} ({urgency})", level.name())
}

fn sample_controller(battery_source: BackendKind, battery: BatteryReading) -> Controller {
    Controller::new(
        "notification-preview",
        "Xbox Wireless Controller",
        ControllerSource::GameInput,
        battery,
    )
    .with_battery(battery_source, battery)
}
