use crate::{AppResult, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    backend::{
        ControllerBattery, ControllerEventInput, ControllerInput, ControllerRumbler,
        GameInputBackend, XInputBackend,
    },
    monitor::ControllerMonitor,
    rumble::BatteryWarningRumbler,
};

pub struct ControllerService<
    N: Notifier,
    I = GameInputBackend,
    B = XInputBackend,
    R = GameInputBackend,
> {
    pub(super) monitor: ControllerMonitor,
    pub(super) input: I,
    pub(super) battery: B,
    pub(super) notifier: N,
    pub(super) rumbler: BatteryWarningRumbler<R>,
    pub(super) config: ControllerServiceConfig,
}

impl<N: Notifier> ControllerService<N, GameInputBackend, XInputBackend, GameInputBackend> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self::with_providers(
            notifier,
            config,
            GameInputBackend::new(),
            XInputBackend::new(),
            GameInputBackend::new(),
        )
    }
}

impl<N, I, B, R> ControllerService<N, I, B, R>
where
    N: Notifier,
    I: ControllerInput + ControllerEventInput,
    B: ControllerBattery,
    R: ControllerRumbler + Clone + Send + 'static,
{
    pub fn with_providers(
        notifier: N,
        config: ControllerServiceConfig,
        input: I,
        battery: B,
        rumbler: R,
    ) -> Self {
        Self {
            monitor: ControllerMonitor::with_warning_policy(config.warning_policy().clone()),
            input,
            battery,
            notifier,
            rumbler: BatteryWarningRumbler::with_backend(config.rumble_config().clone(), rumbler),
            config,
        }
    }

    pub fn run_until_ctrl_c(&mut self) -> AppResult<()> {
        self.run_until_ctrl_c_or(|| false)
    }

    pub fn run_until_ctrl_c_or(&mut self, should_stop: impl Fn() -> bool) -> AppResult<()> {
        self.run_until_ctrl_c_or_reconfigure(should_stop, || Ok(None))
    }

    pub fn run_until_ctrl_c_or_reconfigure(
        &mut self,
        should_stop: impl Fn() -> bool,
        mut next_config: impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<()> {
        let run_state = RunState::with_ctrl_c()?;

        match self.input.start_event_stream() {
            Ok(stream) => {
                if let Err(_event_error) =
                    self.run_backend_event_loop(&run_state, &should_stop, stream, &mut next_config)
                    && run_state.active(&should_stop)
                {
                    self.run_polling_loop(&run_state, &should_stop, &mut next_config)?;
                }
            }
            Err(_start_error) => {
                self.run_polling_loop(&run_state, &should_stop, &mut next_config)?
            }
        }

        Ok(())
    }

    pub fn apply_config(&mut self, config: ControllerServiceConfig) {
        self.monitor
            .set_warning_policy(config.warning_policy().clone());
        self.rumbler.set_config(config.rumble_config().clone());
        self.config = config;
    }

    pub(super) fn apply_pending_config(
        &mut self,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<()> {
        if let Some(config) = next_config()? {
            self.apply_config(config);
        }

        Ok(())
    }
}
