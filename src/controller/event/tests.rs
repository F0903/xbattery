use crate::{
    battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning},
    controller::{Controller, ControllerSource},
    notifier::NotificationUrgency,
};

use super::{ControllerEvent, ControllerNotificationPolicy};

#[test]
fn connected_notifications_are_normal_urgency() {
    let event = ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));

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
    let event = ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
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
