use crate::{
    controller::battery::{BatteryWarning, BatteryWarningReading},
    notifier::{Notification, NotificationUrgency},
};

const BATTERY_STATUS_TITLE: &str = "Xbox Controller Battery Status";

#[derive(Clone, Debug)]
pub struct ControllerNotificationPolicy {
    notify_connected: bool,
    notify_disconnected: bool,
}

impl ControllerNotificationPolicy {
    pub fn new() -> Self {
        Self {
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

    pub(in crate::controller::event) fn notify_connected(&self) -> bool {
        self.notify_connected
    }

    pub(in crate::controller::event) fn notify_disconnected(&self) -> bool {
        self.notify_disconnected
    }

    pub(in crate::controller::event) fn notification_for_battery_warning(
        &self,
        warning: &BatteryWarning,
    ) -> Notification {
        match warning.reading() {
            BatteryWarningReading::Precise(percent) => Notification::with_urgency(
                BATTERY_STATUS_TITLE,
                format!("Battery level is {percent}%"),
                warning_urgency(warning),
            ),
            BatteryWarningReading::Coarse(level) => Notification::with_urgency(
                BATTERY_STATUS_TITLE,
                format!("Battery level is ~{}%", level.estimated_percent()),
                warning_urgency(warning),
            ),
        }
    }
}

impl Default for ControllerNotificationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

fn warning_urgency(warning: &BatteryWarning) -> NotificationUrgency {
    if warning.urgent() {
        NotificationUrgency::Urgent
    } else {
        NotificationUrgency::High
    }
}
