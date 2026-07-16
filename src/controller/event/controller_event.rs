use crate::{
    audio::AudioClip,
    controller::{
        Controller,
        battery::{BatteryReading, BatteryWarning},
    },
    notifier::Notification,
};

use super::{ControllerNotificationPolicy, controller_notification_policy::battery_level_text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerEvent {
    Connected(Controller),
    Disconnected(Controller),
    BatteryStatus {
        controller: Controller,
        warning: Option<BatteryWarning>,
    },
    BatteryWarning {
        warning: BatteryWarning,
    },
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
            Self::BatteryStatus {
                controller,
                warning,
            } => policy.notification_for_battery_status(controller, warning.as_ref()),
            Self::BatteryWarning { warning } if warning.level().notify() => {
                Some(policy.notification_for_battery_warning(warning))
            }
            Self::BatteryWarning { .. } => None,
        }
    }

    pub fn audio(&self) -> Option<&AudioClip> {
        match self {
            Self::BatteryStatus { warning, .. } => {
                warning.as_ref().and_then(|warning| warning.level().audio())
            }
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
