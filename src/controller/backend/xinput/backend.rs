use std::{thread, time::Duration};

use crate::{
    AppResult,
    controller::{
        Controller, ControllerSource,
        backend::{BackendKind, BatteryBackend, InputBackend},
        battery::{BatteryCharge, BatteryLevel, BatteryReading},
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
        settle_battery_readings(Self::battery_readings_once, thread::sleep)
    }
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

fn settle_battery_readings(
    mut read_once: impl FnMut() -> AppResult<Vec<BatteryReading>>,
    mut sleep: impl FnMut(Duration),
) -> AppResult<Vec<BatteryReading>> {
    let mut readings = read_once()?;
    if !should_wait_for_battery_to_settle(&readings) {
        return Ok(readings);
    }

    for _ in 1..BATTERY_SETTLE_ATTEMPTS {
        sleep(BATTERY_SETTLE_DELAY);

        readings = read_once()?;
        if !should_wait_for_battery_to_settle(&readings) {
            break;
        }
    }

    Ok(readings)
}

#[cfg(test)]
mod tests {
    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading};

    use super::{settle_battery_readings, should_wait_for_battery_to_settle};

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
    fn settling_returns_first_non_suspicious_reading() {
        let readings = settle_sequence(vec![
            vec![reading(BatteryLevel::Empty)],
            vec![reading(BatteryLevel::Medium)],
            vec![reading(BatteryLevel::Full)],
        ]);

        assert_eq!(readings, vec![reading(BatteryLevel::Medium)]);
    }

    #[test]
    fn settling_returns_latest_suspicious_reading() {
        let readings = settle_sequence(vec![
            vec![reading(BatteryLevel::Low)],
            vec![reading(BatteryLevel::Empty)],
        ]);

        assert_eq!(readings, vec![reading(BatteryLevel::Empty)]);
    }

    fn reading(level: BatteryLevel) -> BatteryReading {
        BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level))
    }

    fn settle_sequence(mut sequence: Vec<Vec<BatteryReading>>) -> Vec<BatteryReading> {
        sequence.reverse();
        let mut last = sequence.pop().expect("sequence must not be empty");

        settle_battery_readings(
            || {
                if let Some(readings) = sequence.pop() {
                    last = readings;
                }

                Ok(last.clone())
            },
            |_| {},
        )
        .unwrap()
    }
}
