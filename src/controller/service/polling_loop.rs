use std::{thread, time::Duration};

use crate::{
    AppResult,
    controller::backend::{
        ControllerBattery, ControllerEventInput, ControllerInput, ControllerRumbler,
    },
    notifier::Notifier,
};

use super::{ControllerService, ControllerServiceConfig, run_state::RunState};

impl<N, I, B, R> ControllerService<N, I, B, R>
where
    N: Notifier,
    I: ControllerInput + ControllerEventInput,
    B: ControllerBattery,
    R: ControllerRumbler + Clone + Send + 'static,
{
    pub(super) fn run_polling_loop(
        &mut self,
        run_state: &RunState,
        should_stop: &impl Fn() -> bool,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<()> {
        self.apply_pending_config(next_config)?;
        self.poll_and_notify()?;

        while run_state.active(should_stop) {
            if !self.wait_for_next_poll(run_state, should_stop, next_config)? {
                break;
            }

            self.apply_pending_config(next_config)?;
            self.poll_and_notify()?;
        }

        Ok(())
    }

    fn wait_for_next_poll(
        &mut self,
        run_state: &RunState,
        should_stop: &impl Fn() -> bool,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<bool> {
        let mut elapsed = Duration::ZERO;

        while elapsed < self.config.poll_interval() {
            self.apply_pending_config(next_config)?;

            if !run_state.active(should_stop) {
                return Ok(false);
            }

            let remaining = self.config.poll_interval() - elapsed;
            let sleep_for = remaining.min(self.config.control_wait_slice());
            thread::sleep(sleep_for);
            elapsed += sleep_for;
        }

        self.apply_pending_config(next_config)?;

        Ok(run_state.active(should_stop))
    }
}
