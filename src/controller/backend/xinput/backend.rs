use crate::{
    AppResult,
    controller::{
        Controller,
        battery::{BatteryKind, BatteryReading},
    },
};

#[cfg(debug_assertions)]
use super::XInputDiagnosticReport;
use super::native;

#[derive(Clone, Copy, Debug, Default)]
pub struct XInputBackend;

impl XInputBackend {
    #[cfg(debug_assertions)]
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

    fn battery_readings() -> AppResult<Vec<BatteryReading>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
    }

    pub(crate) fn enrich_controllers(&self, controllers: Vec<Controller>) -> Vec<Controller> {
        // GameInput device IDs cannot be correlated with XInput slots.
        if controllers.len() != 1 {
            return controllers;
        }

        let Ok(readings) = Self::battery_readings() else {
            return controllers;
        };

        attach_when_unambiguous(controllers, readings)
    }
}

fn attach_when_unambiguous(
    controllers: Vec<Controller>,
    readings: Vec<BatteryReading>,
) -> Vec<Controller> {
    if controllers.len() != 1 || readings.len() != 1 {
        return controllers;
    }

    controllers
        .into_iter()
        .zip(readings)
        .map(|(controller, reading)| {
            if reading.kind != BatteryKind::Wired
                && reading.charge.is_unknown()
                && !controller.battery().charge.is_unknown()
            {
                controller
            } else {
                controller.with_battery(reading)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::controller::{
        Controller,
        battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
    };

    use super::attach_when_unambiguous;

    #[test]
    fn attaches_matching_battery_readings() {
        let controllers = vec![Controller::new("one", reading(BatteryLevel::Full))];

        let controllers = attach_when_unambiguous(controllers, vec![reading(BatteryLevel::Medium)]);

        assert_eq!(controllers[0].battery(), reading(BatteryLevel::Medium));
    }

    #[test]
    fn leaves_controllers_unchanged_when_reading_count_differs() {
        let controllers = vec![Controller::new("one", reading(BatteryLevel::Full))];

        let controllers = attach_when_unambiguous(controllers, Vec::new());

        assert_eq!(controllers[0].battery(), reading(BatteryLevel::Full));
    }

    #[test]
    fn does_not_replace_a_known_gameinput_charge_with_unknown_xinput_data() {
        let known = reading(BatteryLevel::Full);
        let controllers = vec![Controller::new("one", known)];
        let unknown = BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown);

        let controllers = attach_when_unambiguous(controllers, vec![unknown]);

        assert_eq!(controllers[0].battery(), known);
    }

    #[test]
    fn wired_xinput_data_overrides_a_spurious_gameinput_charge() {
        let controllers = vec![Controller::new("one", reading(BatteryLevel::Empty))];
        let wired = BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown);

        let controllers = attach_when_unambiguous(controllers, vec![wired]);

        assert_eq!(controllers[0].battery(), wired);
    }

    #[test]
    fn does_not_pair_multiple_controllers_by_enumeration_order() {
        let controllers = vec![
            Controller::new("one", reading(BatteryLevel::Full)),
            Controller::new("two", reading(BatteryLevel::Full)),
        ];

        let controllers = attach_when_unambiguous(
            controllers,
            vec![reading(BatteryLevel::Low), reading(BatteryLevel::Medium)],
        );

        assert!(
            controllers
                .iter()
                .all(|controller| controller.battery() == reading(BatteryLevel::Full))
        );
    }

    fn reading(level: BatteryLevel) -> BatteryReading {
        BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level))
    }
}
