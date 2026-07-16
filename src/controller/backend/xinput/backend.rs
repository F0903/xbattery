use crate::{AppResult, controller::Controller};

#[cfg(debug_assertions)]
use super::XInputDiagnosticReport;
use super::native;
use super::snapshot::ControllerSnapshot;

#[cfg(debug_assertions)]
pub(crate) fn diagnostic_reports() -> AppResult<Vec<XInputDiagnosticReport>> {
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

pub(crate) fn poll_controllers() -> AppResult<Vec<Controller>> {
    Ok(native::poll_controllers()?
        .into_iter()
        .flatten()
        .map(controller_from_snapshot)
        .collect())
}

fn controller_from_snapshot(snapshot: ControllerSnapshot) -> Controller {
    Controller::new(format!("xinput:{}", snapshot.slot), snapshot.battery)
}

#[cfg(test)]
mod tests {
    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading};

    use super::controller_from_snapshot;
    use crate::controller::backend::xinput::snapshot::ControllerSnapshot;

    #[test]
    fn polling_identity_is_stable_per_xinput_slot() {
        let battery = reading(BatteryLevel::Medium);

        let controller = controller_from_snapshot(ControllerSnapshot {
            slot: 2,
            packet_number: 123,
            battery,
        });

        assert_eq!(controller.id(), "xinput:2");
        assert_eq!(controller.battery(), battery);
    }

    fn reading(level: BatteryLevel) -> BatteryReading {
        BatteryReading::new(BatteryKind::Alkaline, BatteryCharge::Coarse(level))
    }
}
