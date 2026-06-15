use crate::AppResult;

use super::{
    Controller,
    backend::{BatteryBackend, GameInputBackend, InputBackend, XInputBackend},
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
    I: InputBackend,
    B: BatteryBackend,
{
    pub fn with_providers(input: I, battery: B) -> Self {
        Self { input, battery }
    }

    pub fn poll(&self) -> AppResult<Vec<Controller>> {
        let controllers = self.input.poll_controllers()?;
        Ok(self.battery.attach_to_many(controllers))
    }
}

impl Default for ControllerPoller<GameInputBackend, XInputBackend> {
    fn default() -> Self {
        Self::new()
    }
}
