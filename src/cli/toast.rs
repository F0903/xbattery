use std::{thread, time::Duration};

use xbattery::{
    AppResult,
    battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning},
    config::AppConfig,
    controller::{
        Controller, ControllerSource,
        backend::BackendKind,
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
    let policy =
        ControllerNotificationPolicy::new(config.notifications.urgent_precise_threshold_percent)
            .with_connectivity_notifications(true, true);
    let notifier = ToastNotifier::new(config.toast_config());
    let previews = preview_events();

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
    label: &'static str,
    event: ControllerEvent,
}

fn preview_events() -> Vec<PreviewEvent> {
    vec![
        PreviewEvent {
            label: "connected",
            event: ControllerEvent::Connected(sample_controller(
                BackendKind::XInput,
                BatteryReading::new(
                    BatteryKind::Alkaline,
                    BatteryCharge::Coarse(BatteryLevel::Full),
                ),
            )),
        },
        PreviewEvent {
            label: "disconnected",
            event: ControllerEvent::Disconnected(sample_controller(
                BackendKind::XInput,
                BatteryReading::new(BatteryKind::Disconnected, BatteryCharge::Unknown),
            )),
        },
        precise_warning("50% battery warning", 50),
        precise_warning("25% battery warning", 25),
        precise_warning("10% critical battery warning", 10),
        coarse_warning("medium coarse battery warning", BatteryLevel::Medium),
        coarse_warning("low coarse battery warning", BatteryLevel::Low),
        coarse_warning("empty critical coarse battery warning", BatteryLevel::Empty),
    ]
}

fn precise_warning(label: &'static str, percent: u8) -> PreviewEvent {
    PreviewEvent {
        label,
        event: ControllerEvent::BatteryWarning {
            current: sample_controller(
                BackendKind::GameInput,
                BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Precise(percent)),
            ),
            warning: BatteryWarning::Precise(percent),
        },
    }
}

fn coarse_warning(label: &'static str, level: BatteryLevel) -> PreviewEvent {
    PreviewEvent {
        label,
        event: ControllerEvent::BatteryWarning {
            current: sample_controller(
                BackendKind::XInput,
                BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level)),
            ),
            warning: BatteryWarning::Coarse(level),
        },
    }
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
