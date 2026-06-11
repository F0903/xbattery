use std::{thread, time::Duration};

use crate::{
    battery::{BatteryLevel, BatteryWarning},
    xinput,
};

use super::{Controller, ControllerSource, event::ControllerEvent};

#[derive(Clone, Debug)]
pub struct ControllerRumbler {
    config: ControllerRumbleConfig,
}

impl ControllerRumbler {
    pub fn new(config: ControllerRumbleConfig) -> Self {
        Self { config }
    }

    pub fn rumble_for_event(&self, event: &ControllerEvent) {
        if !self.config.enabled {
            return;
        }

        let Some(pattern) = RumblePattern::for_event(event) else {
            return;
        };

        let Some(slot) = target_slot(event.controller()) else {
            return;
        };

        let config = self.config.clone();
        thread::spawn(move || {
            let _ = run_pattern(slot, pattern, config);
        });
    }
}

#[derive(Clone, Debug)]
pub struct ControllerRumbleConfig {
    pub enabled: bool,
    pub motor_strength_percent: u8,
    pub pulse_duration: Duration,
    pub gap_duration: Duration,
}

impl ControllerRumbleConfig {
    pub fn new(
        enabled: bool,
        motor_strength_percent: u8,
        pulse_duration: Duration,
        gap_duration: Duration,
    ) -> Self {
        Self {
            enabled,
            motor_strength_percent,
            pulse_duration,
            gap_duration,
        }
    }
}

impl Default for ControllerRumbleConfig {
    fn default() -> Self {
        Self::new(
            false,
            35,
            Duration::from_millis(120),
            Duration::from_millis(100),
        )
    }
}

pub fn rumble_single_xinput_controller(
    config: ControllerRumbleConfig,
    pulses: u8,
) -> crate::AppResult<u32> {
    let slot = xinput::single_connected_slot()?
        .ok_or("rumble-test requires exactly one connected XInput controller")?;
    run_pattern(slot, RumblePattern { pulses }, config)?;

    Ok(slot)
}

pub fn rumble_xinput_slot(
    slot: u32,
    config: ControllerRumbleConfig,
    pulses: u8,
) -> crate::AppResult<()> {
    run_pattern(slot, RumblePattern { pulses }, config)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RumblePattern {
    pulses: u8,
}

impl RumblePattern {
    fn for_event(event: &ControllerEvent) -> Option<Self> {
        match event {
            ControllerEvent::BatteryWarning { warning, .. } => Some(Self {
                pulses: pulses_for_warning(*warning),
            }),
            ControllerEvent::Connected(_) | ControllerEvent::Disconnected(_) => None,
        }
    }
}

fn pulses_for_warning(warning: BatteryWarning) -> u8 {
    match warning {
        BatteryWarning::Precise(percent) if percent <= 10 => 3,
        BatteryWarning::Precise(percent) if percent <= 25 => 2,
        BatteryWarning::Precise(_) => 1,
        BatteryWarning::Coarse(BatteryLevel::Empty) => 3,
        BatteryWarning::Coarse(BatteryLevel::Low) => 2,
        BatteryWarning::Coarse(BatteryLevel::Medium) => 1,
        BatteryWarning::Coarse(BatteryLevel::Full) => 0,
    }
}

fn target_slot(controller: &Controller) -> Option<u32> {
    if controller.source() == ControllerSource::XInput {
        return parse_xinput_slot(controller.id());
    }

    xinput::single_connected_slot().ok().flatten()
}

fn parse_xinput_slot(id: &str) -> Option<u32> {
    id.strip_prefix("xinput:")?.parse().ok()
}

fn run_pattern(
    slot: u32,
    pattern: RumblePattern,
    config: ControllerRumbleConfig,
) -> crate::AppResult<()> {
    let strength = motor_speed(config.motor_strength_percent);

    for pulse in 0..pattern.pulses {
        xinput::set_vibration(slot, strength, strength)?;
        thread::sleep(config.pulse_duration);
        xinput::stop_vibration(slot)?;

        if pulse + 1 < pattern.pulses {
            thread::sleep(config.gap_duration);
        }
    }

    Ok(())
}

fn motor_speed(percent: u8) -> u16 {
    ((percent.min(100) as u32 * u16::MAX as u32) / 100) as u16
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        battery::{BatteryCharge, BatteryKind, BatteryReading, BatteryWarning},
        controller::{Controller, ControllerSource},
    };

    use super::{
        ControllerRumbleConfig, ControllerRumbler, RumblePattern, motor_speed, parse_xinput_slot,
        pulses_for_warning,
    };

    #[test]
    fn precise_warning_pulses_scale_by_threshold() {
        assert_eq!(pulses_for_warning(BatteryWarning::Precise(50)), 1);
        assert_eq!(pulses_for_warning(BatteryWarning::Precise(25)), 2);
        assert_eq!(pulses_for_warning(BatteryWarning::Precise(10)), 3);
    }

    #[test]
    fn coarse_warning_pulses_scale_by_level() {
        assert_eq!(
            pulses_for_warning(BatteryWarning::Coarse(crate::battery::BatteryLevel::Medium)),
            1
        );
        assert_eq!(
            pulses_for_warning(BatteryWarning::Coarse(crate::battery::BatteryLevel::Low)),
            2
        );
        assert_eq!(
            pulses_for_warning(BatteryWarning::Coarse(crate::battery::BatteryLevel::Empty)),
            3
        );
    }

    #[test]
    fn ignores_connectivity_events() {
        let event = crate::controller::event::ControllerEvent::Connected(controller());

        assert_eq!(RumblePattern::for_event(&event), None);
    }

    #[test]
    fn parses_xinput_slot_from_controller_id() {
        assert_eq!(parse_xinput_slot("xinput:2"), Some(2));
        assert_eq!(parse_xinput_slot("gameinput:abc"), None);
    }

    #[test]
    fn motor_strength_is_clamped_to_percent() {
        assert_eq!(motor_speed(0), 0);
        assert_eq!(motor_speed(100), u16::MAX);
    }

    #[test]
    fn disabled_rumbler_is_a_noop() {
        let rumbler = ControllerRumbler::new(ControllerRumbleConfig::new(
            false,
            100,
            Duration::from_millis(1),
            Duration::from_millis(1),
        ));
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
}
