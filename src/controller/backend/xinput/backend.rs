use std::{thread, time::Duration};

use crate::{
    AppResult,
    controller::{
        Controller,
        battery::{BatteryCharge, BatteryLevel, BatteryReading},
    },
};

use super::{XInputDiagnosticReport, native};

const BATTERY_SETTLE_ATTEMPTS: usize = 13;
const BATTERY_SETTLE_DELAY: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, Default)]
pub struct XInputBackend;

impl XInputBackend {
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

    fn battery_readings_once() -> AppResult<Vec<BatteryReading>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
    }

    fn settled_battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        settle_battery_readings(Self::battery_readings_once, thread::sleep)
    }

    pub(crate) fn attach_to_many(&self, controllers: Vec<Controller>) -> Vec<Controller> {
        let Ok(readings) = self.settled_battery_readings() else {
            return controllers;
        };

        attach_to_many(controllers, readings)
    }

    pub(crate) fn attach_to_one(&self, controller: Controller) -> Controller {
        let Ok(readings) = self.settled_battery_readings() else {
            return controller;
        };

        attach_to_one(controller, &readings)
    }
}

fn attach_to_many(controllers: Vec<Controller>, readings: Vec<BatteryReading>) -> Vec<Controller> {
    if readings.len() != controllers.len() {
        return controllers;
    }

    controllers
        .into_iter()
        .zip(readings)
        .map(|(controller, reading)| controller.with_battery(reading))
        .collect()
}

fn attach_to_one(controller: Controller, readings: &[BatteryReading]) -> Controller {
    match readings {
        [reading] => controller.with_battery(*reading),
        _ => controller,
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
    use crate::controller::{
        Controller,
        battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
    };

    use super::{
        attach_to_many, attach_to_one, settle_battery_readings, should_wait_for_battery_to_settle,
    };

    #[test]
    fn attaches_matching_battery_readings() {
        let controllers = vec![Controller::new("one", reading(BatteryLevel::Full))];

        let controllers = attach_to_many(controllers, vec![reading(BatteryLevel::Medium)]);

        assert_eq!(controllers[0].battery(), reading(BatteryLevel::Medium));
    }

    #[test]
    fn leaves_controllers_unchanged_when_reading_count_differs() {
        let controllers = vec![Controller::new("one", reading(BatteryLevel::Full))];

        let controllers = attach_to_many(controllers, Vec::new());

        assert_eq!(controllers[0].battery(), reading(BatteryLevel::Full));
    }

    #[test]
    fn attaches_to_one_only_for_a_single_reading() {
        let controller = Controller::new("one", reading(BatteryLevel::Full));

        let attached = attach_to_one(controller.clone(), &[reading(BatteryLevel::Medium)]);
        let unchanged = attach_to_one(controller, &[]);

        assert_eq!(attached.battery(), reading(BatteryLevel::Medium));
        assert_eq!(unchanged.battery(), reading(BatteryLevel::Full));
    }

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
