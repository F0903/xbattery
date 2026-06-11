mod notification_policy;

use crate::{
    battery::{BatteryCharge, BatteryKind, BatteryReading, BatteryWarning},
    notifier::Notification,
};

use super::Controller;

pub use notification_policy::ControllerNotificationPolicy;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerEvent {
    Connected(Controller),
    Disconnected(Controller),
    BatteryWarning {
        current: Controller,
        warning: BatteryWarning,
    },
}

impl ControllerEvent {
    pub fn controller(&self) -> &Controller {
        match self {
            Self::Connected(controller) | Self::Disconnected(controller) => controller,
            Self::BatteryWarning { current, .. } => current,
        }
    }

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
            Self::BatteryWarning {
                current: _,
                warning,
            } => Some(policy.notification_for_battery_warning(*warning)),
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
        Some(level) => format!("Controller has been disconnected. Last known battery level was {level}"),
        None => "Controller has been disconnected".to_string(),
    }
}

fn battery_level_text(battery: BatteryReading) -> Option<String> {
    match battery.charge {
        BatteryCharge::Precise(percent) => Some(format!("{percent}%")),
        BatteryCharge::Coarse(level) => Some(format!("~{}%", level.estimated_percent())),
        BatteryCharge::Unknown if battery.kind == BatteryKind::Wired => Some("wired".to_string()),
        BatteryCharge::Unknown => None,
    }
}

#[cfg(test)]
mod tests;
