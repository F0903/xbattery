mod notification_policy;

use crate::{battery::BatteryWarning, notifier::Notification};

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
                format!("{} connected", controller.name()),
                format!(
                    "Battery: {}. Controller source: {}. Battery source: {}.",
                    controller.battery().description(),
                    controller.source().label(),
                    controller.battery_source().label()
                ),
            )),
            Self::Disconnected(_) if !policy.notify_disconnected() => None,
            Self::Disconnected(controller) => Some(Notification::new(
                format!("{} disconnected", controller.name()),
                "The controller is no longer connected.",
            )),
            Self::BatteryWarning { current, warning } => {
                Some(policy.notification_for_battery_warning(current, *warning))
            }
        }
    }
}

#[cfg(test)]
mod tests;
