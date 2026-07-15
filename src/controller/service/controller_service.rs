use std::{
    collections::HashSet,
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, audio, notifier::Notifier};

use super::{ControllerServiceConfig, run_state::RunState};
use crate::controller::{
    Controller,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DueSnapshot {
    Regular,
    Confirmation,
}

struct PendingBackendEvent {
    controller: Controller,
    is_connected: bool,
    is_device_event: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PostSnapshotDisposition {
    Drop,
    ConnectedMissingFallback,
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
        if let Err(error) = self.poll_and_notify() {
            eprintln!("initial controller snapshot failed: {error}");
        }
        let mut last_poll = Instant::now();

        while run_state.active(should_stop) {
            self.apply_pending_config(next_config)?;
            let regular_poll_delay = self
                .config
                .poll_interval()
                .saturating_sub(last_poll.elapsed());
            let wait_timeout = next_event_wait(
                self.config.control_wait_slice(),
                regular_poll_delay,
                self.monitor.next_confirmation_delay(),
            );

            let event = match stream.recv_timeout(wait_timeout) {
                Ok(event) => Some(self.pending_backend_event(event)),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => {
                    return Ok(EventLoopExit::StreamDisconnected);
                }
            };

            let regular_snapshot_due = last_poll.elapsed() >= self.config.poll_interval();
            let confirmation_snapshot_due = self
                .monitor
                .next_confirmation_delay()
                .is_some_and(|delay| delay.is_zero());
            let regular_snapshot_ids =
                match due_snapshot(regular_snapshot_due, confirmation_snapshot_due) {
                    Some(DueSnapshot::Regular) => {
                        let current_ids = match self.poll_and_notify() {
                            Ok(current_ids) => Some(current_ids),
                            Err(error) => {
                                eprintln!("scheduled controller snapshot failed: {error}");
                                self.monitor.defer_due_confirmations();
                                None
                            }
                        };
                        last_poll = Instant::now();
                        current_ids
                    }
                    Some(DueSnapshot::Confirmation) => {
                        self.refresh_pending_confirmations();
                        None
                    }
                    None => None,
                };

            if let Some(event) = event {
                let refresh_topology = match regular_snapshot_ids.as_ref() {
                    Some(current_ids) => match post_snapshot_disposition(&event, current_ids) {
                        PostSnapshotDisposition::Drop => continue,
                        PostSnapshotDisposition::ConnectedMissingFallback => false,
                    },
                    None => true,
                };

                if self.process_backend_event(event, refresh_topology) {
                    last_poll = Instant::now();
                }
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

    fn poll_and_notify(&mut self) -> AppResult<HashSet<String>> {
        let current = self.input.poll_controllers()?;
        let current_ids = current
            .iter()
            .map(|controller| controller.id().to_string())
            .collect();
        self.observe_current(current);
        Ok(current_ids)
    }

    fn observe_current(&mut self, current: Vec<Controller>) {
        let current = self.battery.enrich_controllers(current);
        let events = self.monitor.observe_current(current);
        self.notify_events(events);
    }

    fn pending_backend_event(&self, event: GameInputEvent) -> PendingBackendEvent {
        let is_device_event = event.is_device_event();
        let (controller, is_connected) = self.input.controller_from_event(event);

        PendingBackendEvent {
            controller,
            is_connected,
            is_device_event,
        }
    }

    fn process_backend_event(
        &mut self,
        event: PendingBackendEvent,
        refresh_topology: bool,
    ) -> bool {
        let PendingBackendEvent {
            controller,
            is_connected,
            is_device_event,
        } = event;

        if is_device_event && refresh_topology {
            match self.input.poll_controllers() {
                Ok(current)
                    if !is_connected
                        || current
                            .iter()
                            .any(|current| current.id() == controller.id()) =>
                {
                    self.observe_current(current);
                    return true;
                }
                Ok(_) => {}
                Err(error) => eprintln!("controller topology refresh failed: {error}"),
            }
        }

        let events = self.monitor.observe_incremental(controller, is_connected);

        self.notify_events(events);
        false
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
) -> Duration {
    next_scheduled_wait(control_wait.min(regular_poll_wait), confirmation_wait)
}

fn due_snapshot(
    regular_snapshot_due: bool,
    confirmation_snapshot_due: bool,
) -> Option<DueSnapshot> {
    if regular_snapshot_due {
        Some(DueSnapshot::Regular)
    } else if confirmation_snapshot_due {
        Some(DueSnapshot::Confirmation)
    } else {
        None
    }
}

fn post_snapshot_disposition(
    event: &PendingBackendEvent,
    current_ids: &HashSet<String>,
) -> PostSnapshotDisposition {
    if event.is_device_event && event.is_connected && !current_ids.contains(event.controller.id()) {
        PostSnapshotDisposition::ConnectedMissingFallback
    } else {
        PostSnapshotDisposition::Drop
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
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
        ControllerService, ControllerServiceConfig, DueSnapshot, PendingBackendEvent,
        PostSnapshotDisposition, due_snapshot, next_event_wait, next_scheduled_wait,
        post_snapshot_disposition,
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
            next_event_wait(Duration::from_millis(250), Duration::from_secs(60), None,),
            Duration::from_millis(250)
        );
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_millis(100),
                Some(Duration::from_secs(6)),
            ),
            Duration::from_millis(100)
        );
        assert_eq!(
            next_event_wait(
                Duration::from_millis(250),
                Duration::from_secs(60),
                Some(Duration::from_millis(50)),
            ),
            Duration::from_millis(50)
        );
    }

    #[test]
    fn due_confirmation_snapshot_is_selected_at_the_event_boundary() {
        assert_eq!(due_snapshot(false, true), Some(DueSnapshot::Confirmation));
        assert_eq!(due_snapshot(false, false), None);
    }

    #[test]
    fn one_regular_snapshot_services_simultaneous_deadlines() {
        assert_eq!(due_snapshot(true, true), Some(DueSnapshot::Regular));
        assert_eq!(due_snapshot(true, false), Some(DueSnapshot::Regular));
    }

    #[test]
    fn successful_snapshot_drops_stale_disconnect_and_reading_events() {
        let current_ids = HashSet::from(["one".to_string()]);
        let disconnected = pending_event("one", true, false);
        let reading = pending_event("one", false, true);

        assert_eq!(
            post_snapshot_disposition(&disconnected, &current_ids),
            PostSnapshotDisposition::Drop
        );
        assert_eq!(
            post_snapshot_disposition(&reading, &current_ids),
            PostSnapshotDisposition::Drop
        );
    }

    #[test]
    fn successful_snapshot_retains_only_a_connected_device_missing_from_topology() {
        let current_ids = HashSet::from(["present".to_string()]);
        let missing_connection = pending_event("missing", true, true);
        let present_connection = pending_event("present", true, true);

        assert_eq!(
            post_snapshot_disposition(&missing_connection, &current_ids),
            PostSnapshotDisposition::ConnectedMissingFallback
        );
        assert_eq!(
            post_snapshot_disposition(&present_connection, &current_ids),
            PostSnapshotDisposition::Drop
        );
    }

    fn pending_event(id: &str, is_device_event: bool, is_connected: bool) -> PendingBackendEvent {
        PendingBackendEvent {
            controller: Controller::new(
                id,
                BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown),
            ),
            is_connected,
            is_device_event,
        }
    }
}
