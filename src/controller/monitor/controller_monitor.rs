use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::controller::{
    battery::{
        BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning,
        BatteryWarningLevel, BatteryWarningPolicy,
    },
    event::ControllerEvent,
};

use super::super::Controller;

// Guarded readings require a fresh sample after this delay before becoming user-visible.
const READING_CONFIRMATION_DELAY: Duration = Duration::from_secs(6);
const MAX_DEFERRED_REFRESHES: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfirmationCandidate {
    Low,
    Wired,
}

#[derive(Clone, Copy, Debug)]
struct PendingConfirmation {
    candidate: ConfirmationCandidate,
    next_check: Instant,
    inconclusive_refreshes: u8,
    missing_refreshes: u8,
}

impl PendingConfirmation {
    fn new(candidate: ConfirmationCandidate, now: Instant) -> Self {
        Self {
            candidate,
            next_check: now + READING_CONFIRMATION_DELAY,
            inconclusive_refreshes: 0,
            missing_refreshes: 0,
        }
    }

    fn is_due(self, now: Instant) -> bool {
        now >= self.next_check
    }

    fn delay(self, now: Instant) -> Duration {
        self.next_check.saturating_duration_since(now)
    }

    fn defer_inconclusive(&mut self, now: Instant) -> bool {
        self.missing_refreshes = 0;
        self.inconclusive_refreshes += 1;
        if self.inconclusive_refreshes > MAX_DEFERRED_REFRESHES {
            return false;
        }

        self.next_check = now + READING_CONFIRMATION_DELAY;
        true
    }

    fn defer_missing(&mut self, now: Instant) -> bool {
        self.inconclusive_refreshes = 0;
        self.missing_refreshes += 1;
        if self.missing_refreshes > MAX_DEFERRED_REFRESHES {
            return false;
        }

        self.next_check = now + READING_CONFIRMATION_DELAY;
        true
    }

    fn observe(&mut self, is_conclusive: bool) {
        self.missing_refreshes = 0;
        if is_conclusive {
            self.inconclusive_refreshes = 0;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ControllerMonitor {
    previous: Vec<Controller>,
    pending_confirmations: HashMap<String, PendingConfirmation>,
    warning_history: HashMap<String, HashMap<String, BatteryWarningLevel>>,
    warning_policy: BatteryWarningPolicy,
}

impl ControllerMonitor {
    pub fn new(warning_policy: BatteryWarningPolicy) -> Self {
        Self {
            previous: Vec::new(),
            pending_confirmations: HashMap::new(),
            warning_history: HashMap::new(),
            warning_policy,
        }
    }

    pub fn set_warning_policy(&mut self, warning_policy: BatteryWarningPolicy) {
        if self.warning_policy != warning_policy {
            self.warning_history.clear();
        }
        self.warning_policy = warning_policy;
    }

    pub fn observe_current(&mut self, current: Vec<Controller>) -> Vec<ControllerEvent> {
        self.observe_current_at(current, Instant::now())
    }

    pub fn observe_pending(&mut self, current: Vec<Controller>) -> Vec<ControllerEvent> {
        self.observe_pending_at(current, Instant::now())
    }

    pub fn next_confirmation_delay(&self) -> Option<Duration> {
        self.next_confirmation_delay_at(Instant::now())
    }

    pub fn defer_due_confirmations(&mut self) {
        self.defer_due_confirmations_at(Instant::now());
    }

    pub fn observe_incremental(
        &mut self,
        controller: Controller,
        is_connected: bool,
    ) -> Vec<ControllerEvent> {
        self.observe_incremental_at(controller, is_connected, Instant::now())
    }

    fn observe_incremental_at(
        &mut self,
        controller: Controller,
        is_connected: bool,
        now: Instant,
    ) -> Vec<ControllerEvent> {
        if is_connected {
            self.observe_connected_at(controller, now)
                .into_iter()
                .collect()
        } else {
            self.observe_disconnected(controller.id())
                .into_iter()
                .collect()
        }
    }

    fn observe_current_at(
        &mut self,
        current: Vec<Controller>,
        now: Instant,
    ) -> Vec<ControllerEvent> {
        let current_ids = current
            .iter()
            .map(|controller| controller.id().to_string())
            .collect::<HashSet<_>>();
        let mut events = current
            .into_iter()
            .filter_map(|controller| self.observe_connected_at(controller, now))
            .collect::<Vec<_>>();
        let disconnected_ids = self
            .previous
            .iter()
            .filter(|controller| !current_ids.contains(controller.id()))
            .map(|controller| controller.id().to_string())
            .collect::<Vec<_>>();

        events.extend(
            disconnected_ids
                .iter()
                .filter_map(|id| self.observe_disconnected(id)),
        );

        events
    }

    fn observe_pending_at(
        &mut self,
        current: Vec<Controller>,
        now: Instant,
    ) -> Vec<ControllerEvent> {
        let mut events = Vec::new();
        let current_ids = current
            .iter()
            .map(|controller| controller.id().to_string())
            .collect::<HashSet<_>>();

        for controller in current {
            if self.pending_confirmations.contains_key(controller.id())
                && let Some(event) = self.observe_connected_at(controller, now)
            {
                events.push(event);
            }
        }

        let mut expired_missing = Vec::new();
        self.pending_confirmations.retain(|id, pending| {
            if current_ids.contains(id.as_str())
                || !pending.is_due(now)
                || pending.defer_missing(now)
            {
                true
            } else {
                expired_missing.push(id.clone());
                false
            }
        });
        events.extend(
            expired_missing
                .iter()
                .filter_map(|id| self.observe_disconnected(id)),
        );

        events
    }

    fn observe_connected_at(
        &mut self,
        controller: Controller,
        now: Instant,
    ) -> Option<ControllerEvent> {
        let id = controller.id().to_string();
        let Some(index) = self
            .previous
            .iter()
            .position(|previous| previous.id() == controller.id())
        else {
            self.warning_history.remove(&id);
            let controller = if let Some(candidate) = confirmation_candidate(controller.battery()) {
                self.pending_confirmations
                    .insert(id, PendingConfirmation::new(candidate, now));
                let kind = match controller.battery().kind {
                    BatteryKind::Wired => BatteryKind::Unknown,
                    kind => kind,
                };
                controller.with_battery(BatteryReading::new(kind, BatteryCharge::Unknown))
            } else {
                controller
            };

            self.previous.push(controller.clone());
            return Some(ControllerEvent::Connected(controller));
        };

        let previous = self.previous[index].clone();
        let is_unknown = is_transient_unknown(controller.battery());
        if let Some(pending) = self.pending_confirmations.get_mut(&id) {
            pending.observe(!is_unknown);
        }
        if is_unknown {
            let abandon = self
                .pending_confirmations
                .get_mut(&id)
                .is_some_and(|pending| pending.is_due(now) && !pending.defer_inconclusive(now));
            if abandon {
                self.pending_confirmations.remove(&id);
            }
            self.previous[index] = preserve_known_battery(&previous, controller);
            return None;
        }

        let needs_confirmation = needs_confirmation(previous.battery(), controller.battery());
        let candidate = confirmation_candidate(controller.battery());
        let confirmed = needs_confirmation
            && candidate.is_some_and(|candidate| {
                self.pending_confirmations
                    .get(&id)
                    .is_some_and(|pending| pending.candidate == candidate && pending.is_due(now))
            });

        if needs_confirmation && !confirmed {
            if let Some(candidate) = candidate {
                self.pending_confirmations
                    .entry(id)
                    .and_modify(|pending| {
                        if pending.candidate != candidate {
                            *pending = PendingConfirmation::new(candidate, now);
                        }
                    })
                    .or_insert_with(|| PendingConfirmation::new(candidate, now));
            }
            return None;
        }

        self.pending_confirmations.remove(&id);
        self.clear_recovered_warnings(&id, controller.battery());
        let warning = self.warning_between(&previous, &controller).or_else(|| {
            (confirmed
                && warning_requires_current_reading(
                    previous.battery().charge,
                    controller.battery().charge,
                ))
            .then(|| {
                self.warning_policy
                    .warning_for_current(controller.battery())
            })
            .flatten()
        });
        let warning = warning.filter(|warning| self.should_emit_warning(&id, warning));
        if let Some(warning) = &warning {
            let level = warning.level().clone();
            self.warning_history
                .entry(id)
                .or_default()
                .insert(level.name().to_string(), level);
        }
        self.previous[index] = controller;

        warning.map(|warning| ControllerEvent::BatteryWarning { warning })
    }

    fn observe_disconnected(&mut self, id: &str) -> Option<ControllerEvent> {
        self.pending_confirmations.remove(id);
        self.warning_history.remove(id);
        let index = self
            .previous
            .iter()
            .position(|previous| previous.id() == id)?;

        Some(ControllerEvent::Disconnected(self.previous.remove(index)))
    }

    fn next_confirmation_delay_at(&self, now: Instant) -> Option<Duration> {
        self.pending_confirmations
            .values()
            .map(|pending| pending.delay(now))
            .min()
    }

    fn defer_due_confirmations_at(&mut self, now: Instant) {
        self.pending_confirmations
            .retain(|_, pending| !pending.is_due(now) || pending.defer_inconclusive(now));
    }

    fn clear_recovered_warnings(&mut self, id: &str, current: BatteryReading) {
        if current.kind == BatteryKind::Wired {
            self.warning_history.remove(id);
            return;
        }

        let Some(current_percent) = current.charge.estimated_percent() else {
            return;
        };
        let Some(history) = self.warning_history.get_mut(id) else {
            return;
        };

        history.retain(|_, level| {
            recovery_threshold_percent(level, current.charge)
                .is_none_or(|threshold| current_percent <= threshold)
        });
        if history.is_empty() {
            self.warning_history.remove(id);
        }
    }

    fn should_emit_warning(&self, id: &str, warning: &BatteryWarning) -> bool {
        self.warning_history
            .get(id)
            .is_none_or(|history| !history.contains_key(warning.level().name()))
    }

    fn warning_between(
        &self,
        previous: &Controller,
        current: &Controller,
    ) -> Option<BatteryWarning> {
        self.warning_policy
            .warning_between(previous.battery(), current.battery())
    }
}

fn preserve_known_battery(previous: &Controller, current: Controller) -> Controller {
    if is_transient_unknown(current.battery())
        && (previous.battery().kind == BatteryKind::Wired
            || !previous.battery().charge.is_unknown())
    {
        current.with_battery(previous.battery())
    } else {
        current
    }
}

fn is_transient_unknown(reading: BatteryReading) -> bool {
    reading.kind != BatteryKind::Wired && reading.charge.is_unknown()
}

fn requires_confirmation(reading: BatteryReading) -> bool {
    confirmation_candidate(reading).is_some()
}

fn confirmation_candidate(reading: BatteryReading) -> Option<ConfirmationCandidate> {
    if reading.kind == BatteryKind::Wired {
        Some(ConfirmationCandidate::Wired)
    } else if matches!(
        reading.charge,
        BatteryCharge::Precise(0..=10)
            | BatteryCharge::Coarse(BatteryLevel::Empty | BatteryLevel::Low)
    ) {
        Some(ConfirmationCandidate::Low)
    } else {
        None
    }
}

fn needs_confirmation(previous: BatteryReading, current: BatteryReading) -> bool {
    if !requires_confirmation(current) {
        return false;
    }

    if current.kind == BatteryKind::Wired {
        return previous.kind != BatteryKind::Wired;
    }

    match (
        previous.charge.estimated_percent(),
        current.charge.estimated_percent(),
    ) {
        (Some(previous), Some(current)) => current < previous,
        (None, Some(_)) => true,
        _ => false,
    }
}

fn warning_requires_current_reading(previous: BatteryCharge, current: BatteryCharge) -> bool {
    previous.is_unknown() || !same_charge_scale(previous, current)
}

fn same_charge_scale(previous: BatteryCharge, current: BatteryCharge) -> bool {
    matches!(
        (previous, current),
        (BatteryCharge::Precise(_), BatteryCharge::Precise(_))
            | (BatteryCharge::Coarse(_), BatteryCharge::Coarse(_))
    )
}

fn recovery_threshold_percent(level: &BatteryWarningLevel, charge: BatteryCharge) -> Option<u8> {
    match charge {
        BatteryCharge::Precise(_) => level
            .precise_threshold_percent()
            .or_else(|| level.coarse_level()?.estimated_percent()),
        BatteryCharge::Coarse(_) => level
            .coarse_level()
            .and_then(BatteryLevel::estimated_percent)
            .or_else(|| level.precise_threshold_percent()),
        BatteryCharge::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::{
        Controller,
        battery::{
            BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarning,
            BatteryWarningLevel, BatteryWarningPolicy,
        },
        event::{ControllerEvent, ControllerNotificationPolicy},
    };

    use std::time::{Duration, Instant};

    use super::{ControllerMonitor, MAX_DEFERRED_REFRESHES, READING_CONFIRMATION_DELAY};

    #[test]
    fn emits_connected_event_for_new_controller() {
        let mut monitor = ControllerMonitor::default();
        let controller = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));

        let events = monitor.observe_current(vec![controller.clone()]);

        assert_eq!(events, vec![ControllerEvent::Connected(controller)]);
    }

    #[test]
    fn emits_disconnected_event_for_missing_controller() {
        let mut monitor = ControllerMonitor::default();
        let controller = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        monitor.observe_current(vec![controller.clone()]);

        let events = monitor.observe_current(Vec::new());

        assert_eq!(events, vec![ControllerEvent::Disconnected(controller)]);
    }

    #[test]
    fn emits_battery_warning_for_decreasing_coarse_level() {
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let medium = controller("one", BatteryCharge::Coarse(BatteryLevel::Medium));
        monitor.observe_current(vec![full]);

        let events = monitor.observe_current(vec![medium]);

        assert_eq!(
            events,
            vec![ControllerEvent::BatteryWarning {
                warning: BatteryWarning::coarse(
                    BatteryLevel::Medium,
                    BatteryWarningLevel::new(
                        "medium",
                        Some(70),
                        Some(BatteryLevel::Medium),
                        true,
                        false,
                        None,
                    ),
                ),
            }]
        );
    }

    #[test]
    fn a_genuine_recovery_allows_the_same_warning_again() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let medium = controller("one", BatteryCharge::Precise(70));
        monitor.observe_current_at(vec![full.clone()], now);

        let first = monitor.observe_current_at(vec![medium.clone()], now + Duration::from_secs(1));
        monitor.observe_current_at(vec![full], now + Duration::from_secs(2));
        let second = monitor.observe_current_at(vec![medium], now + Duration::from_secs(3));

        assert_eq!(first, vec![precise_medium_warning()]);
        assert_eq!(second, vec![precise_medium_warning()]);
    }

    #[test]
    fn replacing_the_policy_resets_warning_history() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        monitor.observe_current_at(vec![controller("one", BatteryCharge::Precise(100))], now);
        assert_eq!(
            monitor.observe_current_at(
                vec![controller("one", BatteryCharge::Precise(40))],
                now + Duration::from_secs(1),
            ),
            vec![precise_low_warning()]
        );
        let replacement =
            BatteryWarningLevel::new("low", Some(20), Some(BatteryLevel::Low), true, false, None);
        monitor.set_warning_policy(BatteryWarningPolicy::new(vec![replacement.clone()]));
        monitor.observe_current_at(
            vec![controller("one", BatteryCharge::Precise(30))],
            now + Duration::from_secs(2),
        );

        let events = monitor.observe_current_at(
            vec![controller("one", BatteryCharge::Precise(19))],
            now + Duration::from_secs(3),
        );

        assert_eq!(
            events,
            vec![ControllerEvent::BatteryWarning {
                warning: BatteryWarning::precise(20, replacement),
            }]
        );
    }

    #[test]
    fn ignores_a_one_off_critical_incremental_reading() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(full.clone(), true, now);

        let events = monitor.observe_incremental_at(critical, true, now + Duration::from_secs(1));
        let recovery = monitor.observe_incremental_at(full, true, now + Duration::from_secs(2));

        assert!(events.is_empty());
        assert!(recovery.is_empty());
    }

    #[test]
    fn warns_when_an_incremental_critical_reading_persists() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(full, true, now);
        let first_seen = now + Duration::from_secs(1);
        monitor.observe_incremental_at(critical.clone(), true, first_seen);

        assert!(
            monitor
                .observe_incremental_at(
                    critical.clone(),
                    true,
                    first_seen + READING_CONFIRMATION_DELAY - Duration::from_millis(1),
                )
                .is_empty()
        );

        let events =
            monitor.observe_incremental_at(critical, true, first_seen + READING_CONFIRMATION_DELAY);

        assert_eq!(events, vec![critical_warning()]);
    }

    #[test]
    fn hides_an_unconfirmed_critical_level_on_initial_connection() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));

        let events = monitor.observe_incremental_at(critical, true, now);

        assert_eq!(
            events,
            vec![ControllerEvent::Connected(controller(
                "one",
                BatteryCharge::Unknown
            ))]
        );
        assert_eq!(
            events[0]
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .body(),
            "Controller is connected"
        );
    }

    #[test]
    fn hides_an_unconfirmed_wired_classification_on_initial_connection() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        let provisional = controller("one", BatteryCharge::Unknown);

        let connected = monitor.observe_incremental_at(wired.clone(), true, now);

        assert_eq!(connected, vec![ControllerEvent::Connected(provisional)]);
        assert_eq!(
            connected[0]
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .body(),
            "Controller is connected"
        );
        assert!(
            monitor
                .observe_pending_at(vec![wired.clone()], now + READING_CONFIRMATION_DELAY)
                .is_empty()
        );
        assert_eq!(
            monitor.observe_incremental_at(
                wired.clone(),
                false,
                now + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
            ),
            vec![ControllerEvent::Disconnected(wired)]
        );
    }

    #[test]
    fn ignores_a_one_off_wired_classification_for_a_known_battery() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        monitor.observe_incremental_at(full.clone(), true, now);

        assert!(
            monitor
                .observe_incremental_at(wired, true, now + Duration::from_secs(1))
                .is_empty()
        );
        assert!(
            monitor
                .observe_incremental_at(full, true, now + Duration::from_secs(2))
                .is_empty()
        );
    }

    #[test]
    fn switching_from_low_to_wired_restarts_the_confirmation_delay() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let low = controller("one", BatteryCharge::Precise(10));
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        monitor.observe_incremental_at(full.clone(), true, now);
        let low_seen = now + Duration::from_secs(1);
        monitor.observe_incremental_at(low, true, low_seen);
        let wired_seen = low_seen + READING_CONFIRMATION_DELAY - Duration::from_millis(100);
        monitor.observe_incremental_at(wired.clone(), true, wired_seen);

        let old_deadline = low_seen + READING_CONFIRMATION_DELAY;
        assert!(
            monitor
                .observe_incremental_at(wired.clone(), true, old_deadline)
                .is_empty()
        );
        assert_eq!(
            monitor.next_confirmation_delay_at(old_deadline),
            Some(READING_CONFIRMATION_DELAY - Duration::from_millis(100))
        );

        let new_deadline = wired_seen + READING_CONFIRMATION_DELAY;
        assert!(
            monitor
                .observe_incremental_at(wired.clone(), true, new_deadline)
                .is_empty()
        );
        assert_eq!(monitor.next_confirmation_delay_at(new_deadline), None);
        assert_eq!(
            monitor.observe_incremental_at(
                wired.clone(),
                false,
                new_deadline + Duration::from_secs(1),
            ),
            vec![ControllerEvent::Disconnected(wired)]
        );
    }

    #[test]
    fn switching_from_wired_to_low_restarts_the_confirmation_delay() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        let low = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(wired, true, now);
        let low_seen = now + READING_CONFIRMATION_DELAY - Duration::from_millis(100);
        monitor.observe_incremental_at(low.clone(), true, low_seen);

        let old_deadline = now + READING_CONFIRMATION_DELAY;
        assert!(
            monitor
                .observe_incremental_at(low.clone(), true, old_deadline)
                .is_empty()
        );
        assert_eq!(
            monitor.next_confirmation_delay_at(old_deadline),
            Some(READING_CONFIRMATION_DELAY - Duration::from_millis(100))
        );

        let new_deadline = low_seen + READING_CONFIRMATION_DELAY;
        assert_eq!(
            monitor.observe_incremental_at(low, true, new_deadline),
            vec![critical_warning()]
        );
    }

    #[test]
    fn transient_unknown_does_not_erase_a_confirmed_wired_classification() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        let unknown = controller("one", BatteryCharge::Unknown);
        monitor.observe_incremental_at(wired.clone(), true, now);
        monitor.observe_incremental_at(wired.clone(), true, now + READING_CONFIRMATION_DELAY);

        assert!(
            monitor
                .observe_incremental_at(
                    unknown,
                    true,
                    now + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
                )
                .is_empty()
        );
        assert_eq!(
            monitor.next_confirmation_delay_at(
                now + READING_CONFIRMATION_DELAY + Duration::from_secs(1)
            ),
            None
        );
        assert_eq!(
            monitor.observe_incremental_at(
                wired.clone(),
                false,
                now + READING_CONFIRMATION_DELAY + Duration::from_secs(2),
            ),
            vec![ControllerEvent::Disconnected(wired)]
        );
    }

    #[test]
    fn queued_unknown_after_a_confirmation_does_not_undo_the_fresh_low_snapshot() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));
        let unknown = controller("one", BatteryCharge::Unknown);
        monitor.observe_incremental_at(critical.clone(), true, now);

        let deadline = now + READING_CONFIRMATION_DELAY;
        assert_eq!(
            monitor.observe_pending_at(vec![critical.clone()], deadline),
            vec![critical_warning()]
        );
        assert!(
            monitor
                .observe_incremental_at(unknown, true, deadline)
                .is_empty()
        );
        assert_eq!(monitor.next_confirmation_delay_at(deadline), None);
        assert_eq!(
            monitor.observe_incremental_at(
                critical.clone(),
                false,
                deadline + Duration::from_secs(1),
            ),
            vec![ControllerEvent::Disconnected(critical)]
        );
    }

    #[test]
    fn warns_once_when_an_initial_critical_level_is_confirmed() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(critical.clone(), true, now);

        let events =
            monitor.observe_pending_at(vec![critical.clone()], now + READING_CONFIRMATION_DELAY);
        let repeated = monitor.observe_incremental_at(
            critical,
            true,
            now + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
        );

        assert_eq!(events, vec![critical_warning()]);
        assert!(repeated.is_empty());
    }

    #[test]
    fn snapshot_observations_filter_a_one_off_critical_level() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_current_at(vec![full.clone()], now);

        let transient = monitor.observe_current_at(vec![critical], now + Duration::from_secs(1));
        let recovery = monitor.observe_current_at(vec![full], now + Duration::from_secs(2));

        assert!(transient.is_empty());
        assert!(recovery.is_empty());
    }

    #[test]
    fn snapshot_observations_warn_for_a_persistent_critical_level() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_current_at(vec![full], now);
        monitor.observe_current_at(vec![critical.clone()], now + Duration::from_secs(1));

        let events = monitor.observe_current_at(
            vec![critical],
            now + Duration::from_secs(1) + READING_CONFIRMATION_DELAY,
        );

        assert_eq!(events, vec![critical_warning()]);
    }

    #[test]
    fn coarse_low_readings_keep_the_existing_confirmation_behavior() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let low = controller("one", BatteryCharge::Coarse(BatteryLevel::Low));
        monitor.observe_current_at(vec![full], now);

        assert!(
            monitor
                .observe_current_at(vec![low.clone()], now + Duration::from_secs(1))
                .is_empty()
        );
        let events = monitor.observe_current_at(
            vec![low],
            now + Duration::from_secs(1) + READING_CONFIRMATION_DELAY,
        );

        assert_eq!(events, vec![coarse_low_warning()]);
    }

    #[test]
    fn coarse_to_precise_transition_preserves_a_noncritical_crossing() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let low = controller("one", BatteryCharge::Precise(40));
        monitor.observe_current_at(vec![full], now);

        let events = monitor.observe_current_at(vec![low], now + Duration::from_secs(1));

        assert_eq!(events, vec![precise_low_warning()]);
    }

    #[test]
    fn changing_precision_does_not_repeat_an_already_emitted_empty_warning() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let coarse_empty = controller("one", BatteryCharge::Coarse(BatteryLevel::Empty));
        let precise_empty = controller("one", BatteryCharge::Precise(9));
        monitor.observe_current_at(vec![coarse_empty.clone()], now);
        assert_eq!(
            monitor.observe_current_at(vec![coarse_empty], now + READING_CONFIRMATION_DELAY,),
            vec![coarse_empty_warning()]
        );
        monitor.set_warning_policy(BatteryWarningPolicy::default());

        assert!(
            monitor
                .observe_current_at(
                    vec![precise_empty.clone()],
                    now + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
                )
                .is_empty()
        );
        assert!(
            monitor
                .observe_current_at(
                    vec![precise_empty],
                    now + Duration::from_secs(1)
                        + READING_CONFIRMATION_DELAY
                        + READING_CONFIRMATION_DELAY,
                )
                .is_empty()
        );
    }

    #[test]
    fn changing_precision_still_emits_a_more_severe_warning() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let coarse_low = controller("one", BatteryCharge::Coarse(BatteryLevel::Low));
        let precise_empty = controller("one", BatteryCharge::Precise(5));
        monitor.observe_current_at(vec![full], now);
        let low_seen = now + Duration::from_secs(1);
        monitor.observe_current_at(vec![coarse_low.clone()], low_seen);
        assert_eq!(
            monitor.observe_current_at(vec![coarse_low], low_seen + READING_CONFIRMATION_DELAY,),
            vec![coarse_low_warning()]
        );
        let empty_seen = low_seen + READING_CONFIRMATION_DELAY + Duration::from_secs(1);
        monitor.observe_current_at(vec![precise_empty.clone()], empty_seen);

        let events = monitor
            .observe_current_at(vec![precise_empty], empty_seen + READING_CONFIRMATION_DELAY);

        assert_eq!(events, vec![critical_warning()]);
    }

    #[test]
    fn custom_cross_scale_mapping_deduplicates_by_level_identity() {
        let now = Instant::now();
        let level = BatteryWarningLevel::new(
            "custom",
            Some(20),
            Some(BatteryLevel::Low),
            true,
            false,
            None,
        );
        let mut monitor = ControllerMonitor::new(BatteryWarningPolicy::new(vec![level.clone()]));
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let coarse_low = controller("one", BatteryCharge::Coarse(BatteryLevel::Low));
        let precise_low = controller("one", BatteryCharge::Precise(10));
        monitor.observe_current_at(vec![full], now);
        let coarse_seen = now + Duration::from_secs(1);
        monitor.observe_current_at(vec![coarse_low.clone()], coarse_seen);
        assert_eq!(
            monitor.observe_current_at(vec![coarse_low], coarse_seen + READING_CONFIRMATION_DELAY,),
            vec![ControllerEvent::BatteryWarning {
                warning: BatteryWarning::coarse(BatteryLevel::Low, level),
            }]
        );
        let precise_seen = coarse_seen + READING_CONFIRMATION_DELAY + Duration::from_secs(1);
        monitor.observe_current_at(vec![precise_low.clone()], precise_seen);

        assert!(
            monitor
                .observe_current_at(vec![precise_low], precise_seen + READING_CONFIRMATION_DELAY,)
                .is_empty()
        );
    }

    #[test]
    fn cross_scale_recovery_allows_the_same_warning_again() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full_coarse = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let low_coarse = controller("one", BatteryCharge::Coarse(BatteryLevel::Low));
        monitor.observe_current_at(vec![full_coarse], now);
        let low_seen = now + Duration::from_secs(1);
        monitor.observe_current_at(vec![low_coarse.clone()], low_seen);
        assert_eq!(
            monitor.observe_current_at(vec![low_coarse], low_seen + READING_CONFIRMATION_DELAY,),
            vec![coarse_low_warning()]
        );
        monitor.observe_current_at(
            vec![controller("one", BatteryCharge::Precise(100))],
            low_seen + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
        );

        let events = monitor.observe_current_at(
            vec![controller("one", BatteryCharge::Precise(40))],
            low_seen + READING_CONFIRMATION_DELAY + Duration::from_secs(2),
        );

        assert_eq!(events, vec![precise_low_warning()]);
    }

    #[test]
    fn recovery_survives_alternating_charge_scales() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full_coarse = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let low_coarse = controller("one", BatteryCharge::Coarse(BatteryLevel::Low));
        monitor.observe_current_at(vec![full_coarse], now);
        let first_low = now + Duration::from_secs(1);
        monitor.observe_current_at(vec![low_coarse.clone()], first_low);
        assert_eq!(
            monitor.observe_current_at(
                vec![low_coarse.clone()],
                first_low + READING_CONFIRMATION_DELAY,
            ),
            vec![coarse_low_warning()]
        );
        monitor.observe_current_at(
            vec![controller("one", BatteryCharge::Precise(100))],
            first_low + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
        );
        let second_low = first_low + READING_CONFIRMATION_DELAY + Duration::from_secs(2);
        monitor.observe_current_at(vec![low_coarse.clone()], second_low);

        let events =
            monitor.observe_current_at(vec![low_coarse], second_low + READING_CONFIRMATION_DELAY);

        assert_eq!(events, vec![coarse_low_warning()]);
    }

    #[test]
    fn unknown_sample_defers_instead_of_discarding_a_pending_confirmation() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        let unknown = controller("one", BatteryCharge::Unknown);
        monitor.observe_current_at(vec![full], now);
        let first_seen = now + Duration::from_secs(1);
        monitor.observe_current_at(vec![critical.clone()], first_seen);

        assert!(
            monitor
                .observe_pending_at(vec![unknown], first_seen + READING_CONFIRMATION_DELAY,)
                .is_empty()
        );
        assert_eq!(
            monitor.next_confirmation_delay_at(first_seen + READING_CONFIRMATION_DELAY),
            Some(READING_CONFIRMATION_DELAY)
        );
        let events = monitor.observe_pending_at(
            vec![critical],
            first_seen + READING_CONFIRMATION_DELAY + READING_CONFIRMATION_DELAY,
        );

        assert_eq!(events, vec![critical_warning()]);
    }

    #[test]
    fn repeated_missing_snapshots_eventually_disconnect_the_controller() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(critical, true, now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES {
            let refresh_at = now
                + Duration::from_secs(READING_CONFIRMATION_DELAY.as_secs() * u64::from(attempt));
            assert!(
                monitor
                    .observe_pending_at(Vec::new(), refresh_at)
                    .is_empty()
            );
        }

        let final_refresh =
            now + READING_CONFIRMATION_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 1);
        assert_eq!(
            monitor.observe_pending_at(Vec::new(), final_refresh),
            vec![ControllerEvent::Disconnected(controller(
                "one",
                BatteryCharge::Unknown
            ))]
        );

        assert_eq!(monitor.next_confirmation_delay_at(final_refresh), None);
    }

    #[test]
    fn unknown_samples_do_not_preload_the_missing_snapshot_budget() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));
        let unknown = controller("one", BatteryCharge::Unknown);
        monitor.observe_incremental_at(critical.clone(), true, now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES {
            let refresh_at = now + READING_CONFIRMATION_DELAY * u32::from(attempt);
            assert!(
                monitor
                    .observe_pending_at(vec![unknown.clone()], refresh_at)
                    .is_empty()
            );
        }

        let missing_at = now + READING_CONFIRMATION_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 1);
        assert!(
            monitor
                .observe_pending_at(Vec::new(), missing_at)
                .is_empty()
        );

        let confirmed_at = missing_at + READING_CONFIRMATION_DELAY;
        assert_eq!(
            monitor.observe_pending_at(vec![critical], confirmed_at),
            vec![critical_warning()]
        );
    }

    #[test]
    fn observed_low_samples_reset_the_missing_snapshot_streak() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(critical.clone(), true, now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES + 1 {
            let refresh_at = now + READING_CONFIRMATION_DELAY * u32::from(attempt);
            assert!(
                monitor
                    .observe_pending_at(Vec::new(), refresh_at)
                    .is_empty()
            );
            assert!(
                monitor
                    .observe_incremental_at(
                        critical.clone(),
                        true,
                        refresh_at + Duration::from_secs(1),
                    )
                    .is_empty()
            );
        }

        let confirmed_at = now + READING_CONFIRMATION_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 2);
        assert_eq!(
            monitor.observe_pending_at(vec![critical], confirmed_at),
            vec![critical_warning()]
        );
    }

    #[test]
    fn repeated_refresh_errors_eventually_abandon_the_candidate() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        monitor.observe_incremental_at(controller("one", BatteryCharge::Precise(10)), true, now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES + 1 {
            let refresh_at = now + READING_CONFIRMATION_DELAY * u32::from(attempt);
            monitor.defer_due_confirmations_at(refresh_at);
        }

        assert_eq!(
            monitor.next_confirmation_delay_at(
                now + READING_CONFIRMATION_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 1),
            ),
            None
        );
    }

    #[test]
    fn disconnect_clears_the_old_confirmation_deadline() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Precise(100));
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_incremental_at(full.clone(), true, now);
        monitor.observe_incremental_at(critical.clone(), true, now + Duration::from_secs(1));

        let disconnected =
            monitor.observe_incremental_at(critical.clone(), false, now + Duration::from_secs(2));
        assert_eq!(
            monitor.next_confirmation_delay_at(now + Duration::from_secs(2)),
            None
        );
        let reconnected_at = now + Duration::from_secs(3);
        let reconnected = monitor.observe_incremental_at(critical.clone(), true, reconnected_at);
        let before_new_deadline = monitor.observe_incremental_at(
            critical.clone(),
            true,
            reconnected_at + READING_CONFIRMATION_DELAY - Duration::from_millis(1),
        );
        let confirmed = monitor.observe_incremental_at(
            critical,
            true,
            reconnected_at + READING_CONFIRMATION_DELAY,
        );

        assert_eq!(disconnected, vec![ControllerEvent::Disconnected(full)]);
        assert_eq!(
            reconnected,
            vec![ControllerEvent::Connected(controller(
                "one",
                BatteryCharge::Unknown
            ))]
        );
        assert!(before_new_deadline.is_empty());
        assert_eq!(confirmed, vec![critical_warning()]);
    }

    #[test]
    fn unknown_snapshot_does_not_erase_the_last_known_level() {
        let mut monitor = ControllerMonitor::default();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let unknown = controller("one", BatteryCharge::Unknown);
        let medium = controller("one", BatteryCharge::Coarse(BatteryLevel::Medium));
        monitor.observe_current(vec![full]);

        assert!(monitor.observe_current(vec![unknown]).is_empty());
        let events = monitor.observe_current(vec![medium]);

        assert!(matches!(
            events.as_slice(),
            [ControllerEvent::BatteryWarning { .. }]
        ));
    }

    fn controller(id: &str, charge: BatteryCharge) -> Controller {
        Controller::new(id, BatteryReading::new(BatteryKind::Unknown, charge))
    }

    fn critical_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: BatteryWarning::precise(
                10,
                BatteryWarningLevel::new(
                    "empty",
                    Some(10),
                    Some(BatteryLevel::Empty),
                    true,
                    true,
                    None,
                ),
            ),
        }
    }

    fn precise_medium_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: BatteryWarning::precise(
                70,
                BatteryWarningLevel::new(
                    "medium",
                    Some(70),
                    Some(BatteryLevel::Medium),
                    true,
                    false,
                    None,
                ),
            ),
        }
    }

    fn precise_low_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: BatteryWarning::precise(
                40,
                BatteryWarningLevel::new(
                    "low",
                    Some(40),
                    Some(BatteryLevel::Low),
                    true,
                    false,
                    None,
                ),
            ),
        }
    }

    fn coarse_low_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: BatteryWarning::coarse(
                BatteryLevel::Low,
                BatteryWarningLevel::new(
                    "low",
                    Some(40),
                    Some(BatteryLevel::Low),
                    true,
                    false,
                    None,
                ),
            ),
        }
    }

    fn coarse_empty_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: BatteryWarning::coarse(
                BatteryLevel::Empty,
                BatteryWarningLevel::new(
                    "empty",
                    Some(10),
                    Some(BatteryLevel::Empty),
                    true,
                    true,
                    None,
                ),
            ),
        }
    }
}
