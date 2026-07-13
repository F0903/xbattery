use std::{thread, time::Duration};

use crate::{
    AppResult,
    config::AppConfig,
    controller::{
        Controller,
        battery::{
            BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning,
            BatteryWarningLevel,
        },
        event::{ControllerEvent, ControllerNotificationPolicy},
    },
    notifier::{Notification, NotificationUrgency, Notifier, ToastNotifier},
};

const PREVIEW_DELAY: Duration = Duration::from_millis(900);

pub(super) fn test() -> AppResult<()> {
    send_test(
        "Toast notifications are working.",
        NotificationUrgency::Normal,
    )
}

pub(super) fn test_high() -> AppResult<()> {
    send_test(
        "High priority toast notifications are working.",
        NotificationUrgency::High,
    )
}

pub(super) fn test_urgent() -> AppResult<()> {
    send_test(
        "Urgent toast notifications are working.",
        NotificationUrgency::Urgent,
    )
}

fn send_test(body: &str, urgency: NotificationUrgency) -> AppResult<()> {
    let config = AppConfig::load()?;
    let notifier = ToastNotifier::new(config.notifications.app_id);
    notifier.notify(&Notification::with_urgency("xbattery", body, urgency))
}

pub(super) fn preview() -> AppResult<()> {
    let config = AppConfig::load()?;
    let policy = ControllerNotificationPolicy::default();
    let notifier = ToastNotifier::new(config.notifications.app_id.clone());
    let warning_levels = config.battery.warning_levels()?;
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
            event: ControllerEvent::Connected(sample_controller(BatteryReading::new(
                BatteryKind::Alkaline,
                BatteryCharge::Coarse(BatteryLevel::Full),
            ))),
        },
        PreviewEvent {
            label: "disconnected".to_string(),
            event: ControllerEvent::Disconnected(sample_controller(BatteryReading::new(
                BatteryKind::Disconnected,
                BatteryCharge::Unknown,
            ))),
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
            let coarse_level = level.coarse_level()?;
            let percent = coarse_level.estimated_percent()?;
            Some((percent, coarse_level, level.clone()))
        })
        .collect::<Vec<_>>();
    coarse_levels.sort_by(|(left, _, _), (right, _, _)| right.cmp(left));

    for (_, coarse_level, level) in coarse_levels {
        previews.push(coarse_warning(coarse_level, level));
    }

    previews
}

fn precise_warning(percent: u8, level: BatteryWarningLevel) -> PreviewEvent {
    PreviewEvent {
        label: warning_label(&level, format!("{percent}% battery warning")),
        event: ControllerEvent::BatteryWarning {
            warning: BatteryWarning::precise(percent, level),
        },
    }
}

fn coarse_warning(coarse_level: BatteryLevel, level: BatteryWarningLevel) -> PreviewEvent {
    PreviewEvent {
        label: warning_label(&level, format!("{coarse_level} coarse battery warning")),
        event: ControllerEvent::BatteryWarning {
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
    let audio = if level.audio().is_some() {
        ", audio"
    } else {
        ""
    };

    format!("{}: {detail} ({urgency}{audio})", level.name())
}

fn sample_controller(battery: BatteryReading) -> Controller {
    Controller::new("notification-preview", battery)
}
