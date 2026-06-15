use std::thread;

use crate::{
    AppResult,
    controller::{
        Controller, ControllerSource,
        backend::{BackendKind, BatteryBackend, InputBackend, RumbleBackend},
        battery::BatteryReading,
        rumble::{RumbleStep, RumbleTarget},
    },
};

use super::{XInputDiagnosticReport, native, snapshot::ControllerSnapshot};

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
        Ok(native::poll_controllers()?
            .into_iter()
            .flatten()
            .map(|snapshot| snapshot.battery)
            .collect())
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
