use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    thread,
    time::Duration,
};

use crate::{AppResult, battery::BatteryWarningPolicy, notifier::Notifier};

use super::{
    backend::{
        BackendEvent, BackendEventStream, ControllerBattery, ControllerEventInput, ControllerInput,
        ControllerRumbler, GameInputBackend, XInputBackend,
    },
    battery_source::{attach_battery_readings, attach_single_battery_reading},
    event::{ControllerEvent, ControllerNotificationPolicy},
    monitor::ControllerMonitor,
    rumble::{BatteryWarningRumbler, ControllerRumbleConfig},
};

pub struct ControllerService<
    N: Notifier,
    I = GameInputBackend,
    B = XInputBackend,
    R = GameInputBackend,
> {
    monitor: ControllerMonitor,
    input: I,
    battery: B,
    notifier: N,
    rumbler: BatteryWarningRumbler<R>,
    config: ControllerServiceConfig,
}

#[derive(Clone, Debug)]
pub struct ControllerServiceConfig {
    poll_interval: Duration,
    control_wait_slice: Duration,
    warning_policy: BatteryWarningPolicy,
    notification_policy: ControllerNotificationPolicy,
    rumble_config: ControllerRumbleConfig,
}

impl ControllerServiceConfig {
    pub fn new(
        poll_interval: Duration,
        control_wait_slice: Duration,
        warning_policy: BatteryWarningPolicy,
        notification_policy: ControllerNotificationPolicy,
        rumble_config: ControllerRumbleConfig,
    ) -> Self {
        Self {
            poll_interval,
            control_wait_slice,
            warning_policy,
            notification_policy,
            rumble_config,
        }
    }
}

impl Default for ControllerServiceConfig {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(60),
            Duration::from_millis(250),
            BatteryWarningPolicy::default(),
            ControllerNotificationPolicy::default(),
            ControllerRumbleConfig::default(),
        )
    }
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
            monitor: ControllerMonitor::with_warning_policy(config.warning_policy.clone()),
            input,
            battery,
            notifier,
            rumbler: BatteryWarningRumbler::with_backend(config.rumble_config.clone(), rumbler),
            config,
        }
    }

    pub fn run_until_ctrl_c(&mut self) -> AppResult<()> {
        self.run_until_ctrl_c_or(|| false)
    }

    pub fn run_until_ctrl_c_or(&mut self, should_stop: impl Fn() -> bool) -> AppResult<()> {
        let running = Arc::new(AtomicBool::new(true));
        let running_signal = Arc::clone(&running);

        ctrlc::set_handler(move || {
            running_signal.store(false, Ordering::SeqCst);
        })?;

        match self.input.start_event_stream() {
            Ok(stream) => {
                if let Err(_event_error) =
                    self.run_backend_event_loop(&running, &should_stop, stream)
                    && active(&running, &should_stop)
                {
                    self.run_polling_loop(&running, &should_stop)?;
                }
            }
            Err(_start_error) => self.run_polling_loop(&running, &should_stop)?,
        }

        Ok(())
    }

    fn run_backend_event_loop(
        &mut self,
        running: &AtomicBool,
        should_stop: &impl Fn() -> bool,
        stream: BackendEventStream,
    ) -> AppResult<()> {
        while active(running, should_stop) {
            match stream.recv_timeout(self.config.control_wait_slice) {
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
        running: &AtomicBool,
        should_stop: &impl Fn() -> bool,
    ) -> AppResult<()> {
        self.poll_and_notify()?;

        while active(running, should_stop) {
            if !self.wait_for_next_poll(running, should_stop) {
                break;
            }

            self.poll_and_notify()?;
        }

        Ok(())
    }

    fn wait_for_next_poll(&self, running: &AtomicBool, should_stop: &impl Fn() -> bool) -> bool {
        let mut elapsed = Duration::ZERO;

        while elapsed < self.config.poll_interval {
            if !active(running, should_stop) {
                return false;
            }

            let remaining = self.config.poll_interval - elapsed;
            let sleep_for = remaining.min(self.config.control_wait_slice);
            thread::sleep(sleep_for);
            elapsed += sleep_for;
        }

        active(running, should_stop)
    }

    fn poll_and_notify(&mut self) -> AppResult<()> {
        let current = self.input.poll_controllers()?;
        let current = attach_battery_readings(current, &self.battery);
        let events = self.monitor.observe_current(current);
        self.notify_events(events)
    }

    fn process_backend_event(&mut self, event: BackendEvent) -> AppResult<()> {
        let (controller, is_connected) = self.input.controller_from_event(event);
        let controller = attach_single_battery_reading(controller, &self.battery);
        let events = self.monitor.observe_incremental(controller, is_connected);

        self.notify_events(events)
    }

    fn notify_events(&self, events: Vec<ControllerEvent>) -> AppResult<()> {
        for event in events {
            self.rumbler.rumble_for_event(&event);

            if let Some(notification) = event.notification(&self.config.notification_policy) {
                self.notifier.notify(&notification)?;
            }
        }

        Ok(())
    }
}

fn active(running: &AtomicBool, should_stop: &impl Fn() -> bool) -> bool {
    running.load(Ordering::SeqCst) && !should_stop()
}
