use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread,
    time::Duration,
};

use crate::{AppResult, battery::BatteryWarningPolicy, gameinput, notifier::Notifier};

use super::{
    event::{ControllerEvent, ControllerNotificationPolicy},
    monitor::ControllerMonitor,
    poller::ControllerPoller,
    rumble::{ControllerRumbleConfig, ControllerRumbler},
};

pub struct ControllerService<N: Notifier> {
    monitor: ControllerMonitor,
    poller: ControllerPoller,
    notifier: N,
    rumbler: ControllerRumbler,
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

impl<N: Notifier> ControllerService<N> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self {
            monitor: ControllerMonitor::with_warning_policy(config.warning_policy.clone()),
            poller: ControllerPoller::new(),
            notifier,
            rumbler: ControllerRumbler::new(config.rumble_config.clone()),
            config,
        }
    }

    pub fn run_until_ctrl_c(&mut self) -> AppResult<()> {
        let running = Arc::new(AtomicBool::new(true));
        let running_signal = Arc::clone(&running);

        ctrlc::set_handler(move || {
            running_signal.store(false, Ordering::SeqCst);
        })?;

        match gameinput::start_callback_watcher() {
            Ok((watcher, receiver)) => {
                if let Err(_event_error) =
                    self.run_gameinput_event_loop(&running, watcher, receiver)
                {
                    if running.load(Ordering::SeqCst) {
                        self.run_polling_loop(&running)?;
                    }
                }
            }
            Err(_start_error) => self.run_polling_loop(&running)?,
        }

        Ok(())
    }

    fn run_gameinput_event_loop(
        &mut self,
        running: &AtomicBool,
        _watcher: gameinput::CallbackWatcher,
        receiver: Receiver<gameinput::GameInputEvent>,
    ) -> AppResult<()> {
        while running.load(Ordering::SeqCst) {
            match receiver.recv_timeout(self.config.control_wait_slice) {
                Ok(event) => self.process_gameinput_event(event)?,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("GameInput callback channel disconnected".into());
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
        let current = self.poller.poll()?;
        let events = self.monitor.observe_current(current);
        self.notify_events(events)
    }

    fn process_gameinput_event(&mut self, event: gameinput::GameInputEvent) -> AppResult<()> {
        let snapshot = event.into_snapshot();
        let is_connected = snapshot.is_connected();
        let controller = self.poller.from_gameinput_event(snapshot);
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
