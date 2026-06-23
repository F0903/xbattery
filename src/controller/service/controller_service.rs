use std::{sync::mpsc::RecvTimeoutError, thread, time::Duration};

use crate::{AppResult, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    backend::{
        BackendEvent, BackendEventStream, BatteryBackend, EventBackend, GameInputBackend,
        InputBackend, XInputBackend,
    },
    event::ControllerEvent,
    monitor::ControllerMonitor,
};

pub struct ControllerService<N: Notifier, I = GameInputBackend, B = XInputBackend> {
    pub(super) monitor: ControllerMonitor,
    pub(super) input: I,
    pub(super) battery: B,
    pub(super) notifier: N,
    pub(super) config: ControllerServiceConfig,
}

impl<N: Notifier> ControllerService<N, GameInputBackend, XInputBackend> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self::with_providers(
            notifier,
            config,
            GameInputBackend::new(),
            XInputBackend::new(),
        )
    }
}

impl<N, I, B> ControllerService<N, I, B>
where
    N: Notifier,
    I: InputBackend + EventBackend,
    B: BatteryBackend,
{
    pub fn with_providers(
        notifier: N,
        config: ControllerServiceConfig,
        input: I,
        battery: B,
    ) -> Self {
        Self {
            monitor: ControllerMonitor::with_warning_policy(config.warning_policy().clone()),
            input,
            battery,
            notifier,
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

    fn run_backend_event_loop(
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

    fn run_polling_loop(
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

    fn poll_and_notify(&mut self) -> AppResult<()> {
        let current = self.input.poll_controllers()?;
        let current = self.battery.attach_to_many(current);
        let events = self.monitor.observe_current(current);
        self.notify_events(events)
    }

    fn process_backend_event(&mut self, event: BackendEvent) -> AppResult<()> {
        let (controller, is_connected) = self.input.controller_from_event(event);
        let controller = self.battery.attach_to_one(controller);
        let events = self.monitor.observe_incremental(controller, is_connected);

        self.notify_events(events)
    }

    fn notify_events(&self, events: Vec<ControllerEvent>) -> AppResult<()> {
        for event in events {
            if let Some(notification) = event.notification(self.config.notification_policy()) {
                self.notifier.notify(&notification)?;
            }
        }

        Ok(())
    }
}
