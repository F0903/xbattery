use crate::{
    battery::{BatteryLevel, BatteryWarning},
    notifier::{Notification, NotificationUrgency},
};

const BATTERY_STATUS_TITLE: &str = "Xbox Controller Battery Status";

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

    pub(super) fn notify_connected(&self) -> bool {
        self.notify_connected
    }

    pub(super) fn notify_disconnected(&self) -> bool {
        self.notify_disconnected
    }

    pub(super) fn notification_for_battery_warning(&self, warning: BatteryWarning) -> Notification {
        match warning {
            BatteryWarning::Precise(percent) => Notification::with_urgency(
                BATTERY_STATUS_TITLE,
                format!("Battery level is {percent}%"),
                self.precise_warning_urgency(percent),
            ),
            BatteryWarning::Coarse(level) => Notification::with_urgency(
                BATTERY_STATUS_TITLE,
                format!("Battery level is ~{}%", level.estimated_percent()),
                self.coarse_warning_urgency(level),
            ),
        }
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
