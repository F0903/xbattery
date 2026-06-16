use std::{thread, time::Duration};

use crate::{
    AppResult,
    controller::{
        Controller, ControllerSource,
        backend::{BackendKind, BatteryBackend, InputBackend, RumbleBackend},
        battery::{BatteryCharge, BatteryLevel, BatteryReading},
        rumble::{RumbleStep, RumbleTarget},
    },
};

use super::{XInputDiagnosticReport, native, snapshot::ControllerSnapshot};

const BATTERY_SETTLE_ATTEMPTS: usize = 13;
const BATTERY_SETTLE_DELAY: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, Default)]
pub struct XInputBackend;

impl XInputBackend {
    pub fn new() -> Self {
        Self
    }

    fn target_slot(&self, target: RumbleTarget) -> AppResult<Option<u32>> {
        match target {
            RumbleTarget::XInputSlot(slot) => Ok(Some(slot)),
            RumbleTarget::SingleController => native::single_connected_slot(),
        }
    }

    pub fn diagnostic_reports(&self) -> AppResult<Vec<XInputDiagnosticReport>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .enumerate()
            .map(|(slot, snapshot)| match snapshot {
                Some(snapshot) => XInputDiagnosticReport {
                    slot: snapshot.slot,
                    packet_number: Some(snapshot.packet_number),
                    battery: Some(snapshot.battery),
                },
                None => XInputDiagnosticReport {
                    slot: slot as u32,
                    packet_number: None,
                    battery: None,
                },
            })
            .collect())
    }

    fn controller_from_snapshot(snapshot: ControllerSnapshot) -> Controller {
        Controller::new(
            format!("xinput:{}", snapshot.slot),
            snapshot.name(),
            ControllerSource::XInput,
            snapshot.battery,
        )
    }

    fn battery_readings_once() -> AppResult<Vec<BatteryReading>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
    }
}

impl InputBackend for XInputBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(Self::controller_from_snapshot)
            .collect())
    }
}

impl BatteryBackend for XInputBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::XInput
    }

    fn battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        Self::battery_readings_once()
    }

    fn settled_battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        let mut best_readings = Self::battery_readings_once()?;
        if !should_wait_for_battery_to_settle(&best_readings) {
            return Ok(best_readings);
        }

        for _ in 1..BATTERY_SETTLE_ATTEMPTS {
            thread::sleep(BATTERY_SETTLE_DELAY);

            let readings = Self::battery_readings_once()?;
            if battery_reading_score(&readings) > battery_reading_score(&best_readings) {
                best_readings = readings;
            }

            if !should_wait_for_battery_to_settle(&best_readings) {
                break;
            }
        }

        Ok(best_readings)
    }
}

impl RumbleBackend for XInputBackend {
    fn rumble(&self, target: RumbleTarget, steps: &[RumbleStep]) -> AppResult<Option<BackendKind>> {
        let Some(slot) = self.target_slot(target)? else {
            return Ok(None);
        };

        for step in steps {
            native::set_vibration(
                slot,
                motor_float_speed(step.low_frequency),
                motor_float_speed(step.high_frequency),
            )?;
            thread::sleep(step.duration);
            native::stop_vibration(slot)?;
        }

        Ok(Some(BackendKind::XInput))
    }
}

fn motor_float_speed(value: f32) -> u16 {
    ((value.clamp(0.0, 1.0) * u16::MAX as f32).round()) as u16
}

fn should_wait_for_battery_to_settle(readings: &[BatteryReading]) -> bool {
    readings.iter().any(is_suspicious_battery_reading)
}

fn is_suspicious_battery_reading(reading: &BatteryReading) -> bool {
    matches!(
        reading.charge,
        BatteryCharge::Coarse(BatteryLevel::Empty | BatteryLevel::Low)
    )
}

fn battery_reading_score(readings: &[BatteryReading]) -> u16 {
    readings.iter().map(single_battery_reading_score).sum()
}

fn single_battery_reading_score(reading: &BatteryReading) -> u16 {
    match reading.charge {
        BatteryCharge::Precise(percent) => percent.into(),
        BatteryCharge::Coarse(level) => level.estimated_percent().into(),
        BatteryCharge::Unknown => 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading};

    use super::{battery_reading_score, should_wait_for_battery_to_settle};

    #[test]
    fn waits_for_any_low_or_empty_battery_reading_to_settle() {
        assert!(should_wait_for_battery_to_settle(&[
            reading(BatteryLevel::Full),
            reading(BatteryLevel::Low),
        ]));

        assert!(should_wait_for_battery_to_settle(&[reading(
            BatteryLevel::Empty
        )]));

        assert!(!should_wait_for_battery_to_settle(&[reading(
            BatteryLevel::Medium
        )]));
    }

    #[test]
    fn battery_reading_score_prefers_higher_total_charge() {
        assert!(
            battery_reading_score(&[reading(BatteryLevel::Full), reading(BatteryLevel::Medium)])
                > battery_reading_score(&[reading(BatteryLevel::Full), reading(BatteryLevel::Low)])
        );
    }

    fn reading(level: BatteryLevel) -> BatteryReading {
        BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level))
    }
}
