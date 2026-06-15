use crate::controller::{
    Controller, ControllerSource,
    battery::{BatteryCharge, BatteryKind, BatteryReading, BatteryWarning},
};

use super::{
    BatteryWarningRumbler, BatteryWarningStage, ControllerRumbleConfig,
    sequence::{motor_speed, rumble_sequence},
};

#[test]
fn precise_warning_patterns_scale_by_threshold() {
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Precise(50)),
        Some(BatteryWarningStage::Medium)
    );
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Precise(25)),
        Some(BatteryWarningStage::Low)
    );
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Precise(10)),
        Some(BatteryWarningStage::Empty)
    );
}

#[test]
fn coarse_warning_patterns_scale_by_level() {
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Coarse(
            crate::controller::battery::BatteryLevel::Medium
        )),
        Some(BatteryWarningStage::Medium)
    );
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Coarse(
            crate::controller::battery::BatteryLevel::Low,
        )),
        Some(BatteryWarningStage::Low)
    );
    assert_eq!(
        BatteryWarningStage::for_warning(BatteryWarning::Coarse(
            crate::controller::battery::BatteryLevel::Empty
        )),
        Some(BatteryWarningStage::Empty)
    );
}

#[test]
fn ignores_connectivity_events() {
    let event = crate::controller::event::ControllerEvent::Connected(controller());

    assert_eq!(BatteryWarningStage::for_event(&event), None);
}

#[test]
fn motor_strength_is_clamped_to_percent() {
    assert_eq!(motor_speed(0), 0);
    assert_eq!(motor_speed(100), u16::MAX);
}

#[test]
fn gradient_patterns_have_expected_shape() {
    let config = ControllerRumbleConfig::default();

    let medium = rumble_sequence(
        config.pattern_for_stage(BatteryWarningStage::Medium),
        &config,
    );
    let low = rumble_sequence(config.pattern_for_stage(BatteryWarningStage::Low), &config);
    let empty = rumble_sequence(
        config.pattern_for_stage(BatteryWarningStage::Empty),
        &config,
    );

    assert_eq!(medium.len(), 5);
    assert_eq!(low.len(), 8);
    assert_eq!(empty.len(), 17);

    assert!(medium[0].low_frequency > 0.0);
    assert_eq!(medium[0].left_trigger, 0.0);
    assert_eq!(medium[0].right_trigger, 0.0);
    assert!(medium[1].left_trigger > medium[1].low_frequency);
    assert!(medium[1].right_trigger > medium[1].high_frequency);
}

#[test]
fn disabled_rumbler_is_a_noop() {
    let rumbler = BatteryWarningRumbler::new(ControllerRumbleConfig::default());
    let event = crate::controller::event::ControllerEvent::BatteryWarning {
        current: controller(),
        warning: BatteryWarning::Precise(10),
    };

    rumbler.rumble_for_event(&event);
}

fn controller() -> Controller {
    Controller::new(
        "xinput:0",
        "Controller",
        ControllerSource::XInput,
        BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Precise(10)),
    )
}
