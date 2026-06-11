use crate::AppResult;

use super::{
    Controller,
    backend::{ControllerBattery, ControllerInput, GameInputBackend, XInputBackend},
    battery_source::attach_battery_readings,
};

#[derive(Clone, Debug)]
pub struct ControllerPoller<I = GameInputBackend, B = XInputBackend> {
    input: I,
    battery: B,
}

impl ControllerPoller<GameInputBackend, XInputBackend> {
    pub fn new() -> Self {
        Self::with_providers(GameInputBackend::new(), XInputBackend::new())
    }
}

impl<I, B> ControllerPoller<I, B>
where
    I: ControllerInput,
    B: ControllerBattery,
{
    pub fn with_providers(input: I, battery: B) -> Self {
        Self { input, battery }
    }

    pub fn poll(&self) -> AppResult<Vec<Controller>> {
        let controllers = self.input.poll_controllers()?;
        Ok(attach_battery_readings(controllers, &self.battery))
    }
}

impl Default for ControllerPoller<GameInputBackend, XInputBackend> {
    fn default() -> Self {
        Self::new()
    }
}
