use std::thread;

use crate::controller::backend::{
    ControllerRumbler, GameInputBackend, RumbleBackend, RumbleTarget,
};

use super::{
    config::{ControllerRumbleConfig, RumblePattern},
    pattern::BatteryWarningStage,
    sequence::rumble_sequence,
};
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
    R: ControllerRumbler + Clone + Send + 'static,
{
    pub fn with_backend(config: ControllerRumbleConfig, backend: R) -> Self {
        Self { config, backend }
    }

    pub fn rumble_for_event(&self, event: &ControllerEvent) {
        if !self.config.enabled {
            return;
        }

        let Some(stage) = BatteryWarningStage::for_event(event) else {
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
    backend: &impl ControllerRumbler,
    target: RumbleTarget,
    stage: BatteryWarningStage,
    config: &ControllerRumbleConfig,
) -> crate::AppResult<Option<RumbleBackend>> {
    let pattern = config.pattern_for_stage(stage);
    run_pattern(backend, target, pattern, config)
}

pub(super) fn run_pattern(
    backend: &impl ControllerRumbler,
    target: RumbleTarget,
    pattern: &RumblePattern,
    config: &ControllerRumbleConfig,
) -> crate::AppResult<Option<RumbleBackend>> {
    let sequence = rumble_sequence(pattern, config);
    backend.rumble(target, &sequence)
}
