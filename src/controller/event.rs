use crate::{
    battery::{BatteryLevel, BatteryWarning},
    notifier::{Notification, NotificationUrgency},
};

use super::Controller;

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
            Self::Connected(_) if !policy.notify_connected => None,
            Self::Connected(controller) => Some(Notification::new(
                format!("{} connected", controller.name()),
                format!(
                    "Battery: {}. Source: {}.",
                    controller.battery().description(),
                    controller.source().label()
                ),
            )),
            Self::Disconnected(_) if !policy.notify_disconnected => None,
            Self::Disconnected(controller) => Some(Notification::new(
                format!("{} disconnected", controller.name()),
                "The controller is no longer connected.",
            )),
            Self::BatteryWarning { current, warning } => Some(match warning {
                BatteryWarning::Precise(percent) => Notification::with_urgency(
                    format!("{} battery {}%", current.name(), percent),
                    format!(
                        "Current level: {}. Source: {}.",
                        current.battery().description(),
                        current.source().label()
                    ),
                    policy.precise_warning_urgency(*percent),
                ),
                BatteryWarning::Coarse(level) => Notification::with_urgency(
                    format!("{} battery {}", current.name(), level),
                    format!(
                        "Estimated level: {}%. Source: XInput coarse battery state.",
                        level.estimated_percent()
                    ),
                    policy.coarse_warning_urgency(*level),
                ),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControllerNotificationPolicy {
    urgent_precise_threshold_percent: u8,
    notify_connected: bool,
    notify_disconnected: bool,
}

impl ControllerNotificationPolicy {
    pub fn new(urgent_precise_threshold_percent: u8) -> Self {
        Self {
            urgent_precise_threshold_percent,
            notify_connected: true,
            notify_disconnected: true,
        }
    }

    pub fn with_connectivity_notifications(
        mut self,
        notify_connected: bool,
        notify_disconnected: bool,
    ) -> Self {
        self.notify_connected = notify_connected;
        self.notify_disconnected = notify_disconnected;
        self
    }

    fn precise_warning_urgency(&self, percent: u8) -> NotificationUrgency {
        if percent <= self.urgent_precise_threshold_percent {
            NotificationUrgency::Urgent
        } else {
            NotificationUrgency::High
        }
    }

    fn coarse_warning_urgency(&self, level: BatteryLevel) -> NotificationUrgency {
        match level {
            BatteryLevel::Empty => NotificationUrgency::Urgent,
            BatteryLevel::Low | BatteryLevel::Medium => NotificationUrgency::High,
            BatteryLevel::Full => NotificationUrgency::Normal,
        }
    }
}

impl Default for ControllerNotificationPolicy {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning},
        controller::{Controller, ControllerSource},
        notifier::NotificationUrgency,
    };

    use super::{ControllerEvent, ControllerNotificationPolicy};

    #[test]
    fn connected_notifications_are_normal_urgency() {
        let event =
            ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));

        assert_eq!(
            event
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .urgency(),
            NotificationUrgency::Normal
        );
    }

    #[test]
    fn connected_notifications_can_be_disabled() {
        let event =
            ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
        let policy =
            ControllerNotificationPolicy::default().with_connectivity_notifications(false, true);

        assert_eq!(event.notification(&policy), None);
    }

    #[test]
    fn disconnected_notifications_can_be_disabled() {
        let event =
            ControllerEvent::Disconnected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
        let policy =
            ControllerNotificationPolicy::default().with_connectivity_notifications(true, false);

        assert_eq!(event.notification(&policy), None);
    }

    #[test]
    fn precise_critical_battery_notifications_are_urgent() {
        let event = ControllerEvent::BatteryWarning {
            current: controller(BatteryCharge::Precise(10)),
            warning: BatteryWarning::Precise(10),
        };

        assert_eq!(
            event
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .urgency(),
            NotificationUrgency::Urgent
        );
    }

    #[test]
    fn precise_noncritical_battery_notifications_are_high_priority() {
        let event = ControllerEvent::BatteryWarning {
            current: controller(BatteryCharge::Precise(25)),
            warning: BatteryWarning::Precise(25),
        };

        assert_eq!(
            event
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .urgency(),
            NotificationUrgency::High
        );
    }

    #[test]
    fn precise_urgency_threshold_can_be_configured() {
        let event = ControllerEvent::BatteryWarning {
            current: controller(BatteryCharge::Precise(20)),
            warning: BatteryWarning::Precise(20),
        };

        assert_eq!(
            event
                .notification(&ControllerNotificationPolicy::new(25))
                .unwrap()
                .urgency(),
            NotificationUrgency::Urgent
        );
    }

    #[test]
    fn coarse_empty_battery_notifications_are_urgent() {
        let event = ControllerEvent::BatteryWarning {
            current: controller(BatteryCharge::Coarse(BatteryLevel::Empty)),
            warning: BatteryWarning::Coarse(BatteryLevel::Empty),
        };

        assert_eq!(
            event
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .urgency(),
            NotificationUrgency::Urgent
        );
    }

    fn controller(charge: BatteryCharge) -> Controller {
        Controller::new(
            "controller",
            "Controller",
            ControllerSource::XInput,
            BatteryReading::new(BatteryKind::Unknown, charge),
        )
    }
}
