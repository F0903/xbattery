use std::thread;

mod native;

use crate::{
    AppResult,
    battery::BatteryReading,
    controller::{Controller, ControllerSource},
    rumble::RumbleStep,
};

use super::{
    BackendKind, ControllerBattery, ControllerInput, ControllerRumbler, RumbleBackend, RumbleTarget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XInputDiagnosticReport {
    pub slot: u32,
    pub packet_number: Option<u32>,
    pub battery: Option<BatteryReading>,
}

impl XInputDiagnosticReport {
    pub fn is_connected(self) -> bool {
        self.packet_number.is_some()
    }
}

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

    fn controller_from_snapshot(snapshot: native::ControllerSnapshot) -> Controller {
        Controller::new(
            format!("xinput:{}", snapshot.slot),
            snapshot.name(),
            ControllerSource::XInput,
            snapshot.battery,
        )
    }
}

impl ControllerInput for XInputBackend {
    fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(Self::controller_from_snapshot)
            .collect())
    }
}

impl ControllerBattery for XInputBackend {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::XInput
    }

    fn battery_readings(&self) -> AppResult<Vec<BatteryReading>> {
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
    }
}

impl ControllerRumbler for XInputBackend {
    fn rumble(
        &self,
        target: RumbleTarget,
        steps: &[RumbleStep],
    ) -> AppResult<Option<RumbleBackend>> {
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

        Ok(Some(RumbleBackend::XInput(slot)))
    }
}

fn motor_float_speed(value: f32) -> u16 {
    ((value.clamp(0.0, 1.0) * u16::MAX as f32).round()) as u16
}
