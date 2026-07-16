use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, audio, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    backend::{
        ControllerBackend, ControllerEventStream, ControllerStreamStatus, WindowsControllerBackend,
    },
    event::ControllerEvent,
    monitor::ControllerMonitor,
};

const TOPOLOGY_SETTLE_DELAY: Duration = Duration::from_millis(250);
const TOPOLOGY_SETTLE_ATTEMPTS: u8 = 3;

pub struct ControllerService<N: Notifier, B: ControllerBackend = WindowsControllerBackend> {
    pub(super) monitor: ControllerMonitor,
    pub(super) backend: B,
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
    Pending,
}

impl<N: Notifier> ControllerService<N> {
    pub fn new(notifier: N, config: ControllerServiceConfig) -> Self {
        Self::with_backend(notifier, config, WindowsControllerBackend)
    }
}

impl<N: Notifier, B: ControllerBackend> ControllerService<N, B> {
    pub(crate) fn with_backend(notifier: N, config: ControllerServiceConfig, backend: B) -> Self {
        Self {
            monitor: ControllerMonitor::new(config.warning_policy().clone()),
            backend,
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

        match self.backend.start_event_stream() {
            Ok(stream) => match self.run_backend_event_loop(
                &run_state,
                &should_stop,
                stream,
                &mut next_config,
            )? {
                EventLoopExit::StreamDisconnected if run_state.active(&should_stop) => {
                    eprintln!("controller event stream disconnected; falling back to polling");
                    self.run_polling_loop(&run_state, &should_stop, &mut next_config)?;
                }
                EventLoopExit::StopRequested | EventLoopExit::StreamDisconnected => {}
            },
            Err(error) => {
                eprintln!("controller event stream unavailable ({error}); falling back to polling");
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
        stream: B::EventStream,
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
                self.monitor.next_refresh_delay(),
                follow_up_refresh_delay,
            );

            let topology_changed = match stream.wait_for_change(wait_timeout) {
                ControllerStreamStatus::Changed => true,
                ControllerStreamStatus::TimedOut => false,
                ControllerStreamStatus::Disconnected => {
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
            let pending_refresh_due = self
                .monitor
                .next_refresh_delay()
                .is_some_and(|delay| delay.is_zero());
            match due_refresh(
                topology_changed || regular_refresh_due || follow_up_refresh_due,
                pending_refresh_due,
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
                Some(DueRefresh::Pending) => self.refresh_pending_readings(),
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
        let mut last_refresh = Instant::now();

        while run_state.active(should_stop) {
            match self.wait_for_next_poll(run_state, should_stop, last_refresh, next_config)? {
                Some(DueRefresh::Current) => {
                    self.try_poll_and_notify("controller refresh failed");
                    last_refresh = Instant::now();
                }
                Some(DueRefresh::Pending) => self.refresh_pending_readings(),
                None => break,
            }
        }

        Ok(())
    }

    fn wait_for_next_poll(
        &mut self,
        run_state: &RunState,
        should_stop: &impl Fn() -> bool,
        last_refresh: Instant,
        next_config: &mut impl FnMut() -> AppResult<Option<ControllerServiceConfig>>,
    ) -> AppResult<Option<DueRefresh>> {
        loop {
            self.apply_pending_config(next_config)?;

            if !run_state.active(should_stop) {
                return Ok(None);
            }

            let regular_wait = self
                .config
                .poll_interval()
                .saturating_sub(last_refresh.elapsed());
            let pending_wait = self.monitor.next_refresh_delay();
            let remaining = next_scheduled_wait(regular_wait, pending_wait);
            if remaining.is_zero() {
                return Ok(due_refresh(
                    regular_wait.is_zero(),
                    pending_wait.is_some_and(|delay| delay.is_zero()),
                ));
            }

            let sleep_for = remaining.min(self.config.control_wait_slice());
            thread::sleep(sleep_for);
        }
    }

    fn refresh_pending_readings(&mut self) {
        match self.backend.poll_controllers() {
            Ok(current) => {
                let events = self.monitor.observe_pending(current);
                self.notify_events(events);
            }
            Err(error) => {
                eprintln!("pending battery refresh failed: {error}");
                let events = self.monitor.defer_due_refreshes();
                self.notify_events(events);
            }
        }
    }

    fn poll_and_notify(&mut self) -> AppResult<()> {
        let current = self.backend.poll_controllers()?;
        let events = self.monitor.observe_current(current);
        self.notify_events(events);
        Ok(())
    }

    fn try_poll_and_notify(&mut self, error_context: &str) -> bool {
        let pending_refresh_due = self
            .monitor
            .next_refresh_delay()
            .is_some_and(|delay| delay.is_zero());

        match self.poll_and_notify() {
            Ok(()) => true,
            Err(error) => {
                eprintln!("{error_context}: {error}");
                if pending_refresh_due {
                    let events = self.monitor.defer_due_refreshes();
                    self.notify_events(events);
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

fn next_scheduled_wait(regular_wait: Duration, pending_wait: Option<Duration>) -> Duration {
    pending_wait.map_or(regular_wait, |pending| regular_wait.min(pending))
}

fn next_event_wait(
    control_wait: Duration,
    regular_poll_wait: Duration,
    pending_wait: Option<Duration>,
    follow_up_wait: Option<Duration>,
) -> Duration {
    let wait = next_scheduled_wait(control_wait.min(regular_poll_wait), pending_wait);
    follow_up_wait.map_or(wait, |follow_up| wait.min(follow_up))
}

fn due_refresh(current_refresh_due: bool, pending_refresh_due: bool) -> Option<DueRefresh> {
    if current_refresh_due {
        Some(DueRefresh::Current)
    } else if pending_refresh_due {
        Some(DueRefresh::Pending)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::VecDeque,
        rc::Rc,
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
            backend::{ControllerBackend, ControllerEventStream, ControllerStreamStatus},
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

    #[derive(Clone, Default)]
    struct RecordingNotifier {
        notifications: Rc<RefCell<Vec<Notification>>>,
    }

    impl Notifier for RecordingNotifier {
        fn notify(&self, notification: &Notification) -> AppResult<()> {
            self.notifications.borrow_mut().push(notification.clone());
            Ok(())
        }
    }

    struct TestBackend;
    struct TestEventStream;

    struct SequenceBackend {
        polls: RefCell<VecDeque<Vec<Controller>>>,
    }

    impl SequenceBackend {
        fn new(polls: impl IntoIterator<Item = Vec<Controller>>) -> Self {
            Self {
                polls: RefCell::new(polls.into_iter().collect()),
            }
        }
    }

    impl ControllerBackend for TestBackend {
        type EventStream = TestEventStream;

        fn start_event_stream(&self) -> AppResult<Self::EventStream> {
            Ok(TestEventStream)
        }

        fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
            Ok(vec![full_controller()])
        }
    }

    impl ControllerBackend for SequenceBackend {
        type EventStream = TestEventStream;

        fn start_event_stream(&self) -> AppResult<Self::EventStream> {
            Ok(TestEventStream)
        }

        fn poll_controllers(&self) -> AppResult<Vec<Controller>> {
            Ok(self
                .polls
                .borrow_mut()
                .pop_front()
                .expect("a scripted controller poll"))
        }
    }

    impl ControllerEventStream for TestEventStream {
        fn wait_for_change(&self, _timeout: Duration) -> ControllerStreamStatus {
            ControllerStreamStatus::TimedOut
        }
    }

    #[test]
    fn service_polls_through_the_backend_contract() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let notifier = FailingNotifier {
            attempts: Arc::clone(&attempts),
        };
        let mut service = ControllerService::with_backend(
            notifier,
            ControllerServiceConfig::default(),
            TestBackend,
        );

        assert!(service.try_poll_and_notify("test controller refresh failed"));
        assert_eq!(attempts.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unknown_connection_is_followed_by_one_settled_battery_notification() {
        let notifier = RecordingNotifier::default();
        let notifications = Rc::clone(&notifier.notifications);
        let full = full_controller();
        let backend =
            SequenceBackend::new([vec![unknown_controller()], vec![full.clone()], vec![full]]);
        let mut service =
            ControllerService::with_backend(notifier, ControllerServiceConfig::default(), backend);

        assert!(service.try_poll_and_notify("initial controller refresh failed"));
        service.refresh_pending_readings();
        service.refresh_pending_readings();

        let notifications = notifications.borrow();
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0].title(), "Xbox Controller Connected");
        assert_eq!(notifications[0].body(), "Controller is connected");
        assert_eq!(notifications[1].title(), "Xbox Controller Battery Status");
        assert_eq!(notifications[1].body(), "Battery level is ~100%");
    }

    #[test]
    fn notification_failures_do_not_abort_event_processing() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let notifier = FailingNotifier {
            attempts: Arc::clone(&attempts),
        };
        let service = ControllerService::new(notifier, ControllerServiceConfig::default());

        service.notify_events(vec![ControllerEvent::Connected(full_controller())]);

        assert_eq!(attempts.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn pending_battery_deadline_preempts_the_regular_poll_interval() {
        assert_eq!(
            next_scheduled_wait(Duration::from_secs(60), Some(Duration::from_millis(250)),),
            Duration::from_millis(250)
        );
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
    fn event_wait_honors_regular_polls_and_pending_battery_deadlines() {
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
                Duration::from_secs(1),
                Duration::from_secs(60),
                Some(Duration::from_millis(250)),
                None,
            ),
            Duration::from_millis(250)
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
    fn due_pending_refresh_is_selected_at_the_event_boundary() {
        assert_eq!(due_refresh(false, true), Some(DueRefresh::Pending));
        assert_eq!(due_refresh(false, false), None);
    }

    #[test]
    fn one_current_refresh_services_simultaneous_deadlines() {
        assert_eq!(due_refresh(true, true), Some(DueRefresh::Current));
        assert_eq!(due_refresh(true, false), Some(DueRefresh::Current));
    }

    fn full_controller() -> Controller {
        Controller::new(
            "one",
            BatteryReading::new(
                BatteryKind::Alkaline,
                BatteryCharge::Coarse(BatteryLevel::Full),
            ),
        )
    }

    fn unknown_controller() -> Controller {
        Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown),
        )
    }
}
