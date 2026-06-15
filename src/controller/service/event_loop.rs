use std::sync::mpsc::RecvTimeoutError;

use crate::{
    AppResult,
    controller::backend::{
        BackendEventStream, ControllerBattery, ControllerEventInput, ControllerInput,
        ControllerRumbler,
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
    pub(super) fn run_backend_event_loop(
        &mut self,
        run_state: &RunState,
        should_stop: &impl Fn() -> bool,
        stream: BackendEventStream,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<()> {
        while run_state.active(should_stop) {
            self.apply_pending_config(next_config)?;

            match stream.recv_timeout(self.config.control_wait_slice()) {
                Ok(event) => self.process_backend_event(event)?,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("controller backend callback channel disconnected".into());
                }
            }
        }

        Ok(())
    }
}
