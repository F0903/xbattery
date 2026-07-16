use crate::{
    controller::{
        Controller,
        battery::{
            BatteryCharge, BatteryKind, BatteryReading, BatteryWarning, BatteryWarningReading,
        },
    },
    notifier::{Notification, NotificationUrgency},
};

const BATTERY_STATUS_TITLE: &str = "Xbox Controller Battery Status";

#[derive(Clone, Debug)]
pub struct ControllerNotificationPolicy {
    notify_connected: bool,
    notify_disconnected: bool,
}

impl ControllerNotificationPolicy {
    pub fn new(notify_connected: bool, notify_disconnected: bool) -> Self {
        Self {
            notify_connected,
            notify_disconnected,
        }
    }

    pub(in crate::controller::event) fn notify_connected(&self) -> bool {
        self.notify_connected
    }

    pub(in crate::controller::event) fn notify_disconnected(&self) -> bool {
        self.notify_disconnected
    }

    pub(in crate::controller::event) fn notification_for_battery_status(
        &self,
        controller: &Controller,
        warning: Option<&BatteryWarning>,
    ) -> Option<Notification> {
        let notifying_warning = warning.filter(|warning| warning.level().notify());
        if notifying_warning.is_none() && !self.notify_connected {
            return None;
        }

        Some(Notification::with_urgency(
            BATTERY_STATUS_TITLE,
            battery_status_body(controller.battery()),
            notifying_warning.map_or(NotificationUrgency::Normal, warning_urgency),
        ))
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
                level.estimated_percent().map_or_else(
                    || "Battery level is unknown".to_string(),
                    |percent| format!("Battery level is ~{percent}%"),
                ),
                warning_urgency(warning),
            ),
        }
    }
}

impl Default for ControllerNotificationPolicy {
    fn default() -> Self {
        Self::new(true, true)
    }
}

fn warning_urgency(warning: &BatteryWarning) -> NotificationUrgency {
    if warning.level().urgent() {
        NotificationUrgency::Urgent
    } else {
        NotificationUrgency::High
    }
}

fn battery_status_body(battery: BatteryReading) -> String {
    battery_level_text(battery).map_or_else(
        || "Battery level is unknown".to_string(),
        |level| format!("Battery level is {level}"),
    )
}

pub(in crate::controller::event) fn battery_level_text(battery: BatteryReading) -> Option<String> {
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
