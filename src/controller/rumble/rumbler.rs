use std::thread;

use crate::controller::{
    backend::{BackendKind, GameInputBackend, RumbleBackend},
    battery::BatteryWarningStage,
    rumble::RumbleTarget,
};

use super::config::ControllerRumbleConfig;
use crate::controller::event::ControllerEvent;

#[derive(Clone, Debug)]
pub struct BatteryWarningRumbler<R = GameInputBackend> {
    config: ControllerRumbleConfig,
    backend: R,
}

impl BatteryWarningRumbler<GameInputBackend> {
    pub fn new(config: ControllerRumbleConfig) -> Self {
        Self::with_backend(config, GameInputBackend::new())
    }
}

impl<R> BatteryWarningRumbler<R>
where
    R: RumbleBackend + Clone + Send + 'static,
{
    pub fn with_backend(config: ControllerRumbleConfig, backend: R) -> Self {
        Self { config, backend }
    }

    pub fn set_config(&mut self, config: ControllerRumbleConfig) {
        self.config = config;
    }

    pub fn rumble_for_event(&self, event: &ControllerEvent) {
        if !self.config.enabled {
            return;
        }

        let Some(stage) = event.battery_warning_stage() else {
            return;
        };

        let target = RumbleTarget::for_controller(event.controller());

        let config = self.config.clone();
        let backend = self.backend.clone();
        thread::spawn(move || {
            let _ = run_stage(&backend, target, stage, &config);
        });
    }
}

pub(super) fn run_stage(
    backend: &impl RumbleBackend,
    target: RumbleTarget,
    stage: BatteryWarningStage,
    config: &ControllerRumbleConfig,
) -> crate::AppResult<Option<BackendKind>> {
    backend.rumble(target, &config.steps_for_stage(stage))
}
