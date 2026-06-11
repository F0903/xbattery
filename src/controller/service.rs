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
        let running = Arc::new(AtomicBool::new(true));
        let running_signal = Arc::clone(&running);

        ctrlc::set_handler(move || {
            running_signal.store(false, Ordering::SeqCst);
        })?;

        match self.input.start_event_stream() {
            Ok(stream) => {
                if let Err(_event_error) = self.run_backend_event_loop(&running, stream)
                    && running.load(Ordering::SeqCst)
                {
                    self.run_polling_loop(&running)?;
                }
            }
            Err(_start_error) => self.run_polling_loop(&running)?,
        }

        Ok(())
    }

    fn run_backend_event_loop(
        &mut self,
        running: &AtomicBool,
        stream: BackendEventStream,
    ) -> AppResult<()> {
        while running.load(Ordering::SeqCst) {
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

    fn run_polling_loop(&mut self, running: &AtomicBool) -> AppResult<()> {
        self.poll_and_notify()?;

        while running.load(Ordering::SeqCst) {
            thread::sleep(self.config.poll_interval);
            if running.load(Ordering::SeqCst) {
                self.poll_and_notify()?;
            }
        }

        Ok(())
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
