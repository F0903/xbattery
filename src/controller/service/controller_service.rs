use std::{sync::mpsc::RecvTimeoutError, thread, time::Duration};

use crate::{AppResult, audio, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    backend::{GameInputBackend, GameInputEvent, GameInputEventStream, XInputBackend},
    event::ControllerEvent,
    monitor::ControllerMonitor,
};

pub struct ControllerService<N: Notifier> {
    pub(super) monitor: ControllerMonitor,
    pub(super) input: GameInputBackend,
    pub(super) battery: XInputBackend,
    pub(super) notifier: N,
    pub(super) config: ControllerServiceConfig,
}

enum EventLoopExit {
    StopRequested,
    StreamDisconnected,
}

impl<N: Notifier> ControllerService<N> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self {
            monitor: ControllerMonitor::new(config.warning_policy().clone()),
            input: GameInputBackend,
            battery: XInputBackend,
            notifier,
            config,
        }
    }

    pub fn run_until_ctrl_c_or_reconfigure(
        &mut self,
        should_stop: impl Fn() -> bool,
        mut next_config: impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<()> {
        let run_state = RunState::new()?;

        match self.input.start_event_stream() {
            Ok(stream) => match self.run_backend_event_loop(
                &run_state,
                &should_stop,
                stream,
                &mut next_config,
            )? {
                EventLoopExit::StreamDisconnected if run_state.active(&should_stop) => {
                    eprintln!("GameInput event stream disconnected; falling back to polling");
                    self.run_polling_loop(&run_state, &should_stop, &mut next_config)?;
                }
                EventLoopExit::StopRequested | EventLoopExit::StreamDisconnected => {}
            },
            Err(error) => {
                eprintln!("GameInput event stream unavailable ({error}); falling back to polling");
                self.run_polling_loop(&run_state, &should_stop, &mut next_config)?
            }
        }

        Ok(())
    }

    fn apply_config(&mut self, config: ControllerServiceConfig) {
        self.monitor
            .set_warning_policy(config.warning_policy().clone());
        self.config = config;
    }

    fn apply_pending_config(
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
        stream: GameInputEventStream,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<EventLoopExit> {
        while run_state.active(should_stop) {
            self.apply_pending_config(next_config)?;
            let wait_timeout = next_scheduled_wait(
                self.config.control_wait_slice(),
                self.monitor.next_confirmation_delay(),
            );

            match stream.recv_timeout(wait_timeout) {
                Ok(event) => self.process_backend_event(event),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Ok(EventLoopExit::StreamDisconnected);
                }
            }

            if self
                .monitor
                .next_confirmation_delay()
                .is_some_and(|delay| delay.is_zero())
            {
                self.refresh_pending_confirmations();
            }
        }

        Ok(EventLoopExit::StopRequested)
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

        loop {
            self.apply_pending_config(next_config)?;

            if !run_state.active(should_stop) {
                return Ok(false);
            }

            let remaining = next_scheduled_wait(
                self.config.poll_interval().saturating_sub(elapsed),
                self.monitor.next_confirmation_delay(),
            );
            if remaining.is_zero() {
                break;
            }

            let sleep_for = remaining.min(self.config.control_wait_slice());
            thread::sleep(sleep_for);
            elapsed += sleep_for;
        }

        self.apply_pending_config(next_config)?;

        Ok(run_state.active(should_stop))
    }

    fn refresh_pending_confirmations(&mut self) {
        match self.input.poll_controllers() {
            Ok(current) => {
                let current = self.battery.enrich_controllers(current);
                let events = self.monitor.observe_pending(current);
                self.notify_events(events);
            }
            Err(error) => {
                eprintln!("battery confirmation refresh failed: {error}");
                self.monitor.defer_due_confirmations();
            }
        }
    }

    fn poll_and_notify(&mut self) -> AppResult<()> {
        let current = self.input.poll_controllers()?;
        let current = self.battery.enrich_controllers(current);
        let events = self.monitor.observe_current(current);
        self.notify_events(events);
        Ok(())
    }

    fn process_backend_event(&mut self, event: GameInputEvent) {
        let (controller, is_connected) = self.input.controller_from_event(event);
        let events = self.monitor.observe_incremental(controller, is_connected);

        self.notify_events(events);
    }

    fn notify_events(&self, events: Vec<ControllerEvent>) {
        for event in events {
            self.play_event_sound(&event);

            if let Some(notification) = event.notification(self.config.notification_policy())
                && let Err(error) = self.notifier.notify(&notification)
            {
                eprintln!("notification delivery failed: {error}");
            }
        }
    }

    fn play_event_sound(&self, event: &ControllerEvent) {
        let Some(audio_clip) = event.audio() else {
            return;
        };

        if let Err(error) = audio::play(audio_clip) {
            eprintln!("configured audio playback failed: {error}");
        }
    }
}

fn next_scheduled_wait(regular_wait: Duration, confirmation_wait: Option<Duration>) -> Duration {
    confirmation_wait.map_or(regular_wait, |confirmation| regular_wait.min(confirmation))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use crate::{
        AppResult,
        controller::{
            Controller,
            battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
            event::ControllerEvent,
        },
        notifier::{Notification, Notifier},
    };

    use super::{ControllerService, ControllerServiceConfig, next_scheduled_wait};

    #[derive(Clone)]
    struct FailingNotifier {
        attempts: Arc<AtomicUsize>,
    }

    impl Notifier for FailingNotifier {
        fn notify(&self, _notification: &Notification) -> AppResult<()> {
            self.attempts.fetch_add(1, Ordering::Relaxed);
            Err("test notification failure".into())
        }
    }

    #[test]
    fn notification_failures_do_not_abort_event_processing() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let notifier = FailingNotifier {
            attempts: Arc::clone(&attempts),
        };
        let service = ControllerService::new(notifier, ControllerServiceConfig::default());
        let controller = Controller::new(
            "one",
            BatteryReading::new(
                BatteryKind::Alkaline,
                BatteryCharge::Coarse(BatteryLevel::Full),
            ),
        );

        service.notify_events(vec![ControllerEvent::Connected(controller)]);

        assert_eq!(attempts.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn confirmation_deadline_preempts_the_regular_poll_interval() {
        assert_eq!(
            next_scheduled_wait(Duration::from_secs(60), Some(Duration::from_secs(6))),
            Duration::from_secs(6)
        );
        assert_eq!(
            next_scheduled_wait(Duration::from_millis(250), Some(Duration::ZERO)),
            Duration::ZERO
        );
    }
}
