use crate::{
    audio::AudioClip,
    controller::{
        Controller,
        battery::{BatteryCharge, BatteryKind, BatteryReading, BatteryWarning},
    },
    notifier::Notification,
};

use super::ControllerNotificationPolicy;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerEvent {
    Connected(Controller),
    Disconnected(Controller),
    BatteryWarning { warning: BatteryWarning },
}

impl ControllerEvent {
    pub fn notification(&self, policy: &ControllerNotificationPolicy) -> Option<Notification> {
        match self {
            Self::Connected(_) if !policy.notify_connected() => None,
            Self::Connected(controller) => Some(Notification::new(
                "Xbox Controller Connected",
                connected_body(controller.battery()),
            )),
            Self::Disconnected(_) if !policy.notify_disconnected() => None,
            Self::Disconnected(controller) => Some(Notification::new(
                "Xbox Controller Disconnected",
                disconnected_body(controller.battery()),
            )),
            Self::BatteryWarning { warning } if warning.level().notify() => {
                Some(policy.notification_for_battery_warning(warning))
            }
            Self::BatteryWarning { .. } => None,
        }
    }

    pub fn audio(&self) -> Option<&AudioClip> {
        match self {
            Self::BatteryWarning { warning } => warning.level().audio(),
            Self::Connected(_) | Self::Disconnected(_) => None,
        }
    }
}

fn connected_body(battery: BatteryReading) -> String {
    match battery_level_text(battery) {
        Some(level) => format!("Battery level is {level}"),
        None => "Controller is connected".to_string(),
    }
}

fn disconnected_body(battery: BatteryReading) -> String {
    match battery_level_text(battery) {
        Some(level) => {
            format!("Controller has been disconnected. Last known battery level was {level}")
        }
        None => "Controller has been disconnected".to_string(),
    }
}

fn battery_level_text(battery: BatteryReading) -> Option<String> {
    if battery.kind == BatteryKind::Wired {
        return Some("wired".to_string());
    }

    match battery.charge {
        BatteryCharge::Precise(percent) => Some(format!("{percent}%")),
        BatteryCharge::Coarse(level) => level
            .estimated_percent()
            .map(|percent| format!("~{percent}%")),
        BatteryCharge::Unknown => None,
    }
}
