use crate::controller::battery::BatteryReading;

use super::raw;

#[derive(Clone, Debug)]
pub struct GameInputDiagnosticSnapshot {
    pub timestamp: u64,
    pub source: &'static str,
    pub current_status: String,
    pub previous_status: String,
    pub battery: BatteryReading,
    pub battery_status: &'static str,
    pub remaining_capacity: f32,
    pub full_charge_capacity: f32,
    pub charge_rate: f32,
}

impl GameInputDiagnosticSnapshot {
    pub(super) fn from_event(event: raw::GameInputEvent) -> Self {
        let source = event.source_label();
        Self::from_snapshot(source, event.into_snapshot())
    }

    pub(super) fn from_snapshot(
        source: &'static str,
        snapshot: raw::GameInputDeviceSnapshot,
    ) -> Self {
        Self {
            timestamp: snapshot.timestamp,
            source,
            current_status: snapshot.current_status_description(),
            previous_status: snapshot.previous_status_description(),
            battery: snapshot.battery,
            battery_status: snapshot.battery_status_description(),
            remaining_capacity: snapshot.raw_battery.remaining_capacity,
            full_charge_capacity: snapshot.raw_battery.full_charge_capacity,
            charge_rate: snapshot.raw_battery.charge_rate,
        }
    }
}
