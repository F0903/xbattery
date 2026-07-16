use std::{
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, audio, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    backend::{GameInputBackend, GameInputEventStream, XInputBackend},
    event::ControllerEvent,
    monitor::ControllerMonitor,
};

const TOPOLOGY_SETTLE_DELAY: Duration = Duration::from_millis(250);
const TOPOLOGY_SETTLE_ATTEMPTS: u8 = 3;

pub struct ControllerService<N: Notifier> {
    pub(super) monitor: ControllerMonitor,
    pub(super) events: GameInputBackend,
    pub(super) snapshots: XInputBackend,
    pub(super) notifier: N,
    pub(super) config: ControllerServiceConfig,
}

enum EventLoopExit {
    StopRequested,
    StreamDisconnected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DueRefresh {
    Current,
    Confirmation,
}

impl<N: Notifier> ControllerService<N> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self {
            monitor: ControllerMonitor::new(config.warning_policy().clone()),
            events: GameInputBackend,
            snapshots: XInputBackend,
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

        match self.events.start_event_stream() {
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
        let initial_refresh_failed = !self.try_poll_and_notify("initial controller refresh failed");
        let mut last_refresh = Instant::now();
        let mut follow_up_refresh_at =
            initial_refresh_failed.then(|| Instant::now() + TOPOLOGY_SETTLE_DELAY);
        let mut follow_up_attempts_remaining = if initial_refresh_failed {
            TOPOLOGY_SETTLE_ATTEMPTS
        } else {
            0
        };

        while run_state.active(should_stop) {
            self.apply_pending_config(next_config)?;
            let regular_poll_delay = self
                .config
                .poll_interval()
                .saturating_sub(last_refresh.elapsed());
            let follow_up_refresh_delay = follow_up_refresh_at
                .map(|deadline| deadline.saturating_duration_since(Instant::now()));
            let wait_timeout = next_event_wait(
                self.config.control_wait_slice(),
                regular_poll_delay,
                self.monitor.next_confirmation_delay(),
                follow_up_refresh_delay,
            );

            let topology_changed = match stream.recv_timeout(wait_timeout) {
                Ok(_) => true,
                Err(RecvTimeoutError::Timeout) => false,
                Err(RecvTimeoutError::Disconnected) => {
                    return Ok(EventLoopExit::StreamDisconnected);
                }
            };
            if topology_changed {
                follow_up_refresh_at = Some(Instant::now() + TOPOLOGY_SETTLE_DELAY);
                follow_up_attempts_remaining = TOPOLOGY_SETTLE_ATTEMPTS;
            }

            let regular_refresh_due = last_refresh.elapsed() >= self.config.poll_interval();
            let follow_up_refresh_due =
                follow_up_refresh_at.is_some_and(|deadline| Instant::now() >= deadline);
            let confirmation_refresh_due = self
                .monitor
                .next_confirmation_delay()
                .is_some_and(|delay| delay.is_zero());
            match due_refresh(
                topology_changed || regular_refresh_due || follow_up_refresh_due,
                confirmation_refresh_due,
            ) {
                Some(DueRefresh::Current) => {
                    let refresh_succeeded = self.try_poll_and_notify("controller refresh failed");
                    if follow_up_refresh_due {
                        if refresh_succeeded || follow_up_attempts_remaining <= 1 {
                            follow_up_refresh_at = None;
                            follow_up_attempts_remaining = 0;
                        } else {
                            follow_up_attempts_remaining -= 1;
                            follow_up_refresh_at = Some(Instant::now() + TOPOLOGY_SETTLE_DELAY);
                        }
                    }
                    last_refresh = Instant::now();
                }
                Some(DueRefresh::Confirmation) => self.refresh_pending_confirmations(),
                None => {}
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
        self.try_poll_and_notify("initial controller refresh failed");

        while run_state.active(should_stop) {
            if !self.wait_for_next_poll(run_state, should_stop, next_config)? {
                break;
            }

            self.apply_pending_config(next_config)?;
            self.try_poll_and_notify("controller refresh failed");
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
        match self.snapshots.poll_controllers() {
            Ok(current) => {
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
        let current = self.snapshots.poll_controllers()?;
        let events = self.monitor.observe_current(current);
        self.notify_events(events);
        Ok(())
    }

    fn try_poll_and_notify(&mut self, error_context: &str) -> bool {
        let confirmation_due = self
            .monitor
            .next_confirmation_delay()
            .is_some_and(|delay| delay.is_zero());

        match self.poll_and_notify() {
            Ok(()) => true,
            Err(error) => {
                eprintln!("{error_context}: {error}");
                if confirmation_due {
                    self.monitor.defer_due_confirmations();
                }
                false
            }
        }
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

fn next_event_wait(
    control_wait: Duration,
    regular_poll_wait: Duration,
    confirmation_wait: Option<Duration>,
    follow_up_wait: Option<Duration>,
) -> Duration {
    let wait = next_scheduled_wait(control_wait.min(regular_poll_wait), confirmation_wait);
    follow_up_wait.map_or(wait, |follow_up| wait.min(follow_up))
}

fn due_refresh(current_refresh_due: bool, confirmation_refresh_due: bool) -> Option<DueRefresh> {
    if current_refresh_due {
        Some(DueRefresh::Current)
    } else if confirmation_refresh_due {
        Some(DueRefresh::Confirmation)
    } else {
        None
    }
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

    use super::{
        ControllerService, ControllerServiceConfig, DueRefresh, due_refresh, next_event_wait,
        next_scheduled_wait,
    };

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

    #[test]
    fn event_wait_honors_regular_polls_and_confirmation_deadlines() {
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_secs(60),
                None,
                None,
            ),
            Duration::from_millis(250)
        );
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_millis(100),
                Some(Duration::from_secs(6)),
                None,
            ),
            Duration::from_millis(100)
        );
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_secs(60),
                Some(Duration::from_millis(50)),
                None,
            ),
            Duration::from_millis(50)
        );
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_secs(60),
                None,
                Some(Duration::from_millis(25)),
            ),
            Duration::from_millis(25)
        );
    }

    #[test]
    fn due_confirmation_refresh_is_selected_at_the_event_boundary() {
        assert_eq!(due_refresh(false, true), Some(DueRefresh::Confirmation));
        assert_eq!(due_refresh(false, false), None);
    }

    #[test]
    fn one_current_refresh_services_simultaneous_deadlines() {
        assert_eq!(due_refresh(true, true), Some(DueRefresh::Current));
        assert_eq!(due_refresh(true, false), Some(DueRefresh::Current));
    }
}
