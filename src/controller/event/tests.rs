use crate::{
    controller::{
        Controller, ControllerSource,
        battery::{
            BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning,
            BatteryWarningLevel,
        },
    },
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
fn connected_notifications_use_user_facing_copy() {
    let event = ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.title(), "Xbox Controller Connected");
    assert_eq!(notification.body(), "Battery level is ~100%");
    assert!(!notification.body().contains("XInput"));
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
fn disconnected_notifications_use_user_facing_copy() {
    let event = ControllerEvent::Disconnected(controller(BatteryCharge::Coarse(BatteryLevel::Low)));
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.title(), "Xbox Controller Disconnected");
    assert_eq!(
        notification.body(),
        "Controller has been disconnected. Last known battery level was ~40%"
    );
    assert!(!notification.body().contains("XInput"));
}

#[test]
fn precise_critical_battery_notifications_are_urgent() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Precise(10)),
        warning: precise_warning(10, true),
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
fn precise_battery_notifications_use_user_facing_copy() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Precise(50)),
        warning: precise_warning(50, false),
    };
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.title(), "Xbox Controller Battery Status");
    assert_eq!(notification.body(), "Battery level is 50%");
    assert!(!notification.body().contains("XInput"));
}

#[test]
fn precise_noncritical_battery_notifications_are_high_priority() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Precise(25)),
        warning: precise_warning(25, false),
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
fn battery_warning_urgency_can_be_configured_per_level() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Precise(20)),
        warning: precise_warning(20, true),
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
fn coarse_battery_notifications_use_user_facing_copy() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Coarse(BatteryLevel::Medium)),
        warning: coarse_warning(BatteryLevel::Medium, false),
    };
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.title(), "Xbox Controller Battery Status");
    assert_eq!(notification.body(), "Battery level is ~70%");
    assert!(!notification.body().contains("XInput"));
}

#[test]
fn coarse_empty_battery_notifications_are_urgent() {
    let event = ControllerEvent::BatteryWarning {
        current: controller(BatteryCharge::Coarse(BatteryLevel::Empty)),
        warning: coarse_warning(BatteryLevel::Empty, true),
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

fn precise_warning(percent: u8, urgent: bool) -> BatteryWarning {
    BatteryWarning::precise(
        percent,
        BatteryWarningLevel::new(format!("{percent}%"), Some(percent), None, urgent),
    )
}

fn coarse_warning(level: BatteryLevel, urgent: bool) -> BatteryWarning {
    BatteryWarning::coarse(
        level,
        BatteryWarningLevel::new(level.to_string(), None, Some(level), urgent),
    )
}
