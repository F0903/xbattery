use crate::{
    audio::AudioClip,
    controller::{
        Controller,
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
fn connected_notifications_do_not_invent_a_percentage_for_unknown_levels() {
    let event =
        ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Unknown)));
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.body(), "Controller is connected");
}

#[test]
fn connected_notifications_can_be_disabled() {
    let event = ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
    let policy = ControllerNotificationPolicy::new(false, true);

    assert_eq!(event.notification(&policy), None);
}

#[test]
fn ordinary_battery_status_notifications_use_the_accepted_reading() {
    let readings = [
        (
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Precise(50)),
            "Battery level is 50%",
        ),
        (
            BatteryReading::new(
                BatteryKind::Unknown,
                BatteryCharge::Coarse(BatteryLevel::Medium),
            ),
            "Battery level is ~70%",
        ),
        (
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
            "Battery level is wired",
        ),
    ];

    for (reading, expected_body) in readings {
        let event = ControllerEvent::BatteryStatus {
            controller: controller_with_reading(reading),
            warning: None,
        };
        let notification = event
            .notification(&ControllerNotificationPolicy::default())
            .unwrap();

        assert_eq!(notification.title(), "Xbox Controller Battery Status");
        assert_eq!(notification.body(), expected_body);
        assert_eq!(notification.urgency(), NotificationUrgency::Normal);
    }
}

#[test]
fn battery_status_warning_overrides_connected_gating_and_exposes_audio() {
    let warning = BatteryWarning::precise(
        10,
        BatteryWarningLevel::new(
            "critical",
            Some(10),
            None,
            true,
            true,
            Some(AudioClip::file("critical.wav")),
        ),
    );
    let event = ControllerEvent::BatteryStatus {
        controller: controller(BatteryCharge::Precise(10)),
        warning: Some(warning),
    };
    let notification = event
        .notification(&ControllerNotificationPolicy::new(false, true))
        .unwrap();

    assert_eq!(notification.urgency(), NotificationUrgency::Urgent);
    assert_eq!(event.audio(), Some(&AudioClip::file("critical.wav")));
}

#[test]
fn battery_status_uses_the_actual_precise_value_instead_of_the_warning_threshold() {
    let event = ControllerEvent::BatteryStatus {
        controller: controller(BatteryCharge::Precise(23)),
        warning: Some(BatteryWarning::precise(
            25,
            BatteryWarningLevel::new("low", Some(25), None, true, false, None),
        )),
    };
    let notification = event
        .notification(&ControllerNotificationPolicy::default())
        .unwrap();

    assert_eq!(notification.body(), "Battery level is 23%");
    assert!(!notification.body().contains("25%"));
}

#[test]
fn ordinary_battery_status_honors_connected_notification_gating() {
    let event = ControllerEvent::BatteryStatus {
        controller: controller(BatteryCharge::Precise(50)),
        warning: None,
    };

    assert_eq!(
        event.notification(&ControllerNotificationPolicy::new(false, true)),
        None
    );
}

#[test]
fn notification_disabled_warning_uses_ordinary_status_semantics() {
    let warning = BatteryWarning::precise(
        25,
        BatteryWarningLevel::new(
            "low",
            Some(25),
            None,
            false,
            true,
            Some(AudioClip::file("low.wav")),
        ),
    );
    let event = ControllerEvent::BatteryStatus {
        controller: controller(BatteryCharge::Precise(23)),
        warning: Some(warning),
    };
    let notification = event
        .notification(&ControllerNotificationPolicy::new(true, true))
        .unwrap();

    assert_eq!(notification.urgency(), NotificationUrgency::Normal);
    assert_eq!(notification.body(), "Battery level is 23%");
    assert_eq!(event.audio(), Some(&AudioClip::file("low.wav")));
    assert_eq!(
        event.notification(&ControllerNotificationPolicy::new(false, true)),
        None
    );
}

#[test]
fn disconnected_notifications_can_be_disabled() {
    let event =
        ControllerEvent::Disconnected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));
    let policy = ControllerNotificationPolicy::new(true, false);

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
fn battery_warning_notifications_can_be_disabled_per_level() {
    let event = ControllerEvent::BatteryWarning {
        warning: BatteryWarning::precise(
            25,
            BatteryWarningLevel::new(
                "low",
                Some(25),
                None,
                false,
                false,
                Some(AudioClip::file("low.wav")),
            ),
        ),
    };

    assert_eq!(
        event.notification(&ControllerNotificationPolicy::default()),
        None
    );
}

#[test]
fn battery_warning_events_expose_configured_audio() {
    let event = ControllerEvent::BatteryWarning {
        warning: BatteryWarning::precise(
            25,
            BatteryWarningLevel::new(
                "low",
                Some(25),
                None,
                false,
                false,
                Some(AudioClip::file("low.wav")),
            ),
        ),
    };

    assert_eq!(event.audio(), Some(&AudioClip::file("low.wav")));
}

#[test]
fn connectivity_events_do_not_expose_audio() {
    let event = ControllerEvent::Connected(controller(BatteryCharge::Coarse(BatteryLevel::Full)));

    assert_eq!(event.audio(), None);
}

#[test]
fn battery_warning_urgency_can_be_configured_per_level() {
    let event = ControllerEvent::BatteryWarning {
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
    controller_with_reading(BatteryReading::new(BatteryKind::Unknown, charge))
}

fn controller_with_reading(reading: BatteryReading) -> Controller {
    Controller::new("controller", reading)
}

fn precise_warning(percent: u8, urgent: bool) -> BatteryWarning {
    BatteryWarning::precise(
        percent,
        BatteryWarningLevel::new(
            format!("{percent}%"),
            Some(percent),
            None,
            true,
            urgent,
            None,
        ),
    )
}

fn coarse_warning(level: BatteryLevel, urgent: bool) -> BatteryWarning {
    BatteryWarning::coarse(
        level,
        BatteryWarningLevel::new(level.to_string(), None, Some(level), true, urgent, None),
    )
}
