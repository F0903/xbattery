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
const INITIAL_READING_RETRY_DELAY: Duration = Duration::from_millis(250);
const MAX_DEFERRED_REFRESHES: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfirmationCandidate {
    Low,
    Wired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingReadingKind {
    AwaitingUsable,
    Confirming(ConfirmationCandidate),
}

impl PendingReadingKind {
    fn retry_delay(self) -> Duration {
        match self {
            Self::AwaitingUsable => INITIAL_READING_RETRY_DELAY,
            Self::Confirming(_) => READING_CONFIRMATION_DELAY,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingAnnouncement {
    Connected,
    BatteryStatus,
}

#[derive(Clone, Copy, Debug)]
struct PendingReading {
    kind: PendingReadingKind,
    next_check: Instant,
    inconclusive_refreshes: u8,
    missing_refreshes: u8,
}

impl PendingReading {
    fn awaiting_usable(now: Instant) -> Self {
        Self::new(PendingReadingKind::AwaitingUsable, now)
    }

    fn confirming(candidate: ConfirmationCandidate, now: Instant) -> Self {
        Self::new(PendingReadingKind::Confirming(candidate), now)
    }

    fn new(kind: PendingReadingKind, now: Instant) -> Self {
        Self {
            kind,
            next_check: now + kind.retry_delay(),
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

        self.next_check = now + self.kind.retry_delay();
        true
    }

    fn defer_missing(&mut self, now: Instant) -> bool {
        self.inconclusive_refreshes = 0;
        self.missing_refreshes += 1;
        if self.missing_refreshes > MAX_DEFERRED_REFRESHES {
            return false;
        }

        self.next_check = now + self.kind.retry_delay();
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
    pending_readings: HashMap<String, PendingReading>,
    pending_announcements: HashMap<String, PendingAnnouncement>,
    warning_history: HashMap<String, HashMap<String, BatteryWarningLevel>>,
    warning_policy: BatteryWarningPolicy,
}

impl ControllerMonitor {
    pub fn new(warning_policy: BatteryWarningPolicy) -> Self {
        Self {
            previous: Vec::new(),
            pending_readings: HashMap::new(),
            pending_announcements: HashMap::new(),
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

    pub fn next_refresh_delay(&self) -> Option<Duration> {
        self.next_refresh_delay_at(Instant::now())
    }

    pub fn defer_due_refreshes(&mut self) -> Vec<ControllerEvent> {
        self.defer_due_refreshes_at(Instant::now())
    }

    #[cfg(test)]
    fn observe_incremental_at(
        &mut self,
        controller: Controller,
        is_connected: bool,
        now: Instant,
    ) -> Vec<ControllerEvent> {
        if is_connected {
            self.observe_connected_at(controller, now)
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
            .flat_map(|controller| self.observe_connected_at(controller, now))
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
            if self.pending_readings.contains_key(controller.id()) {
                events.extend(self.observe_connected_at(controller, now));
            }
        }

        let mut expired_missing = Vec::new();
        self.pending_readings.retain(|id, pending| {
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
    ) -> Vec<ControllerEvent> {
        let id = controller.id().to_string();
        let Some(index) = self
            .previous
            .iter()
            .position(|previous| previous.id() == controller.id())
        else {
            self.warning_history.remove(&id);
            let controller = if let Some(candidate) = confirmation_candidate(controller.battery()) {
                self.pending_readings
                    .insert(id.clone(), PendingReading::confirming(candidate, now));
                self.pending_announcements
                    .insert(id.clone(), PendingAnnouncement::Connected);
                let kind = match controller.battery().kind {
                    BatteryKind::Wired => BatteryKind::Unknown,
                    kind => kind,
                };
                controller.with_battery(BatteryReading::new(kind, BatteryCharge::Unknown))
            } else if is_transient_unknown(controller.battery()) {
                self.pending_readings
                    .insert(id.clone(), PendingReading::awaiting_usable(now));
                self.pending_announcements
                    .insert(id.clone(), PendingAnnouncement::BatteryStatus);
                controller
            } else {
                controller
            };

            self.previous.push(controller.clone());
            return match self.pending_announcements.get(&id) {
                Some(PendingAnnouncement::Connected) => Vec::new(),
                Some(PendingAnnouncement::BatteryStatus) | None => {
                    vec![ControllerEvent::Connected(controller)]
                }
            };
        };

        let previous = self.previous[index].clone();
        let is_unknown = is_transient_unknown(controller.battery());
        if let Some(pending) = self.pending_readings.get_mut(&id) {
            pending.observe(!is_unknown);
        }
        if is_unknown {
            let abandon = self
                .pending_readings
                .get_mut(&id)
                .is_some_and(|pending| pending.is_due(now) && !pending.defer_inconclusive(now));
            if abandon {
                self.pending_readings.remove(&id);
            }
            self.previous[index] = preserve_known_battery(&previous, controller);
            return if abandon {
                self.release_pending_connected(&id).into_iter().collect()
            } else {
                Vec::new()
            };
        }

        let needs_confirmation = needs_confirmation(previous.battery(), controller.battery());
        let candidate = confirmation_candidate(controller.battery());
        let confirmed = needs_confirmation
            && candidate.is_some_and(|candidate| {
                self.pending_readings.get(&id).is_some_and(|pending| {
                    pending.kind == PendingReadingKind::Confirming(candidate) && pending.is_due(now)
                })
            });

        if needs_confirmation && !confirmed {
            if let Some(candidate) = candidate {
                self.pending_readings
                    .entry(id)
                    .and_modify(|pending| {
                        if pending.kind != PendingReadingKind::Confirming(candidate) {
                            *pending = PendingReading::confirming(candidate, now);
                        }
                    })
                    .or_insert_with(|| PendingReading::confirming(candidate, now));
            }
            return Vec::new();
        }

        self.pending_readings.remove(&id);
        self.clear_recovered_warnings(&id, controller.battery());
        let warning = self
            .warning_policy
            .warning_between(previous.battery(), controller.battery())
            .or_else(|| {
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
                .entry(id.clone())
                .or_default()
                .insert(level.name().to_string(), level);
        }
        self.previous[index] = controller.clone();

        match self.pending_announcements.remove(&id) {
            Some(PendingAnnouncement::Connected) => {
                let mut events = vec![ControllerEvent::Connected(controller)];
                events.extend(warning.map(|warning| ControllerEvent::BatteryWarning { warning }));
                events
            }
            Some(PendingAnnouncement::BatteryStatus) => {
                vec![ControllerEvent::BatteryStatus {
                    controller,
                    warning,
                }]
            }
            None => warning
                .map(|warning| ControllerEvent::BatteryWarning { warning })
                .into_iter()
                .collect(),
        }
    }

    fn observe_disconnected(&mut self, id: &str) -> Option<ControllerEvent> {
        self.pending_readings.remove(id);
        self.pending_announcements.remove(id);
        self.warning_history.remove(id);
        let index = self
            .previous
            .iter()
            .position(|previous| previous.id() == id)?;

        Some(ControllerEvent::Disconnected(self.previous.remove(index)))
    }

    fn next_refresh_delay_at(&self, now: Instant) -> Option<Duration> {
        self.pending_readings
            .values()
            .map(|pending| pending.delay(now))
            .min()
    }

    fn defer_due_refreshes_at(&mut self, now: Instant) -> Vec<ControllerEvent> {
        let mut abandoned = Vec::new();
        self.pending_readings.retain(|id, pending| {
            let keep = !pending.is_due(now) || pending.defer_inconclusive(now);
            if !keep {
                abandoned.push(id.clone());
            }
            keep
        });

        abandoned
            .into_iter()
            .filter_map(|id| self.release_pending_connected(&id))
            .collect()
    }

    fn release_pending_connected(&mut self, id: &str) -> Option<ControllerEvent> {
        if self.pending_announcements.get(id) != Some(&PendingAnnouncement::Connected) {
            return None;
        }
        self.pending_announcements.remove(id);

        self.previous
            .iter()
            .find(|controller| controller.id() == id)
            .cloned()
            .map(ControllerEvent::Connected)
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
    if confirmation_candidate(current).is_none() {
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

    use super::{
        ControllerMonitor, INITIAL_READING_RETRY_DELAY, MAX_DEFERRED_REFRESHES,
        READING_CONFIRMATION_DELAY,
    };

    #[test]
    fn emits_connected_event_for_new_controller() {
        let mut monitor = ControllerMonitor::default();
        let controller = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));

        let events = monitor.observe_current(vec![controller.clone()]);

        assert_eq!(events, vec![ControllerEvent::Connected(controller)]);
    }

    #[test]
    fn unknown_initial_reading_emits_a_generic_connected_event_and_schedules_a_retry() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);

        let events = monitor.observe_current_at(vec![unknown.clone()], now);

        assert_eq!(events, vec![ControllerEvent::Connected(unknown)]);
        assert_eq!(
            monitor.next_refresh_delay_at(now),
            Some(INITIAL_READING_RETRY_DELAY)
        );
    }

    #[test]
    fn first_usable_reading_after_unknown_emits_one_battery_status() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        monitor.observe_current_at(vec![unknown], now);

        let status =
            monitor.observe_current_at(vec![full.clone()], now + INITIAL_READING_RETRY_DELAY);
        let repeated = monitor.observe_current_at(
            vec![full.clone()],
            now + INITIAL_READING_RETRY_DELAY + Duration::from_secs(1),
        );

        assert_eq!(
            status,
            vec![ControllerEvent::BatteryStatus {
                controller: full,
                warning: None,
            }]
        );
        assert!(repeated.is_empty());
        assert_eq!(
            monitor.next_refresh_delay_at(now + INITIAL_READING_RETRY_DELAY),
            None
        );
    }

    #[test]
    fn unknown_initial_reading_gets_three_accelerated_retries() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        monitor.observe_current_at(vec![unknown.clone()], now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES {
            let refresh_at = now + INITIAL_READING_RETRY_DELAY * u32::from(attempt);
            assert!(
                monitor
                    .observe_pending_at(vec![unknown.clone()], refresh_at)
                    .is_empty()
            );
            assert_eq!(
                monitor.next_refresh_delay_at(refresh_at),
                Some(INITIAL_READING_RETRY_DELAY)
            );
        }

        let final_retry = now + INITIAL_READING_RETRY_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 1);
        assert!(
            monitor
                .observe_pending_at(vec![unknown], final_retry)
                .is_empty()
        );
        assert_eq!(monitor.next_refresh_delay_at(final_retry), None);
    }

    #[test]
    fn retry_exhaustion_keeps_the_later_battery_status_announcement() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        monitor.observe_current_at(vec![unknown.clone()], now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES + 1 {
            let refresh_at = now + INITIAL_READING_RETRY_DELAY * u32::from(attempt);
            assert!(
                monitor
                    .observe_pending_at(vec![unknown.clone()], refresh_at)
                    .is_empty()
            );
        }

        let later = now + Duration::from_secs(60);
        assert_eq!(
            monitor.observe_current_at(vec![full.clone()], later),
            vec![ControllerEvent::BatteryStatus {
                controller: full,
                warning: None,
            }]
        );
    }

    #[test]
    fn initial_unknown_to_low_folds_the_warning_into_one_confirmed_status() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let critical = controller("one", BatteryCharge::Precise(10));
        monitor.observe_current_at(vec![unknown], now);
        let low_seen = now + INITIAL_READING_RETRY_DELAY;

        assert!(
            monitor
                .observe_current_at(vec![critical.clone()], low_seen)
                .is_empty()
        );
        assert!(
            monitor
                .observe_pending_at(
                    vec![critical.clone()],
                    low_seen + READING_CONFIRMATION_DELAY - Duration::from_millis(1),
                )
                .is_empty()
        );

        assert_eq!(
            monitor.observe_pending_at(
                vec![critical.clone()],
                low_seen + READING_CONFIRMATION_DELAY,
            ),
            vec![ControllerEvent::BatteryStatus {
                controller: critical,
                warning: Some(critical_warning_value()),
            }]
        );
    }

    #[test]
    fn initial_unknown_to_wired_waits_for_confirmation_before_status() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let wired = wired_controller("one");
        monitor.observe_current_at(vec![unknown], now);
        let wired_seen = now + INITIAL_READING_RETRY_DELAY;

        assert!(
            monitor
                .observe_current_at(vec![wired.clone()], wired_seen)
                .is_empty()
        );
        assert_eq!(
            monitor
                .observe_pending_at(vec![wired.clone()], wired_seen + READING_CONFIRMATION_DELAY,),
            vec![ControllerEvent::BatteryStatus {
                controller: wired,
                warning: None,
            }]
        );
    }

    #[test]
    fn usable_wireless_reading_supersedes_pending_wired_status_immediately() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let wired = wired_controller("one");
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        monitor.observe_current_at(vec![unknown], now);
        monitor.observe_current_at(vec![wired], now + INITIAL_READING_RETRY_DELAY);

        assert_eq!(
            monitor.observe_current_at(
                vec![full.clone()],
                now + INITIAL_READING_RETRY_DELAY + Duration::from_millis(250),
            ),
            vec![ControllerEvent::BatteryStatus {
                controller: full,
                warning: None,
            }]
        );
    }

    #[test]
    fn switching_low_to_wired_restarts_unknown_status_confirmation() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let critical = controller("one", BatteryCharge::Precise(10));
        let wired = wired_controller("one");
        monitor.observe_current_at(vec![unknown], now);
        let low_seen = now + INITIAL_READING_RETRY_DELAY;
        monitor.observe_current_at(vec![critical], low_seen);
        let wired_seen = low_seen + READING_CONFIRMATION_DELAY - Duration::from_millis(100);
        monitor.observe_current_at(vec![wired.clone()], wired_seen);

        assert!(
            monitor
                .observe_pending_at(vec![wired.clone()], low_seen + READING_CONFIRMATION_DELAY)
                .is_empty()
        );
        assert_eq!(
            monitor.next_refresh_delay_at(low_seen + READING_CONFIRMATION_DELAY),
            Some(READING_CONFIRMATION_DELAY - Duration::from_millis(100))
        );
        assert_eq!(
            monitor
                .observe_pending_at(vec![wired.clone()], wired_seen + READING_CONFIRMATION_DELAY,),
            vec![ControllerEvent::BatteryStatus {
                controller: wired,
                warning: None,
            }]
        );
    }

    #[test]
    fn disconnect_clears_pending_unknown_status_before_reconnect() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        assert_eq!(
            monitor.observe_current_at(vec![unknown.clone()], now),
            vec![ControllerEvent::Connected(unknown.clone())]
        );

        assert_eq!(
            monitor.observe_current_at(Vec::new(), now + Duration::from_millis(100)),
            vec![ControllerEvent::Disconnected(unknown)]
        );
        assert_eq!(
            monitor.observe_current_at(vec![full.clone()], now + Duration::from_millis(200)),
            vec![ControllerEvent::Connected(full)]
        );
    }

    #[test]
    fn refresh_error_exhaustion_keeps_the_unknown_status_announcement() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let unknown = controller("one", BatteryCharge::Unknown);
        let medium = controller("one", BatteryCharge::Coarse(BatteryLevel::Medium));
        monitor.observe_current_at(vec![unknown], now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES + 1 {
            let refresh_at = now + INITIAL_READING_RETRY_DELAY * u32::from(attempt);
            assert!(monitor.defer_due_refreshes_at(refresh_at).is_empty());
        }

        assert_eq!(
            monitor.observe_current_at(vec![medium.clone()], now + Duration::from_secs(60)),
            vec![ControllerEvent::BatteryStatus {
                controller: medium,
                warning: None,
            }]
        );
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
    fn defers_the_connected_event_for_an_unconfirmed_critical_level() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let critical = controller("one", BatteryCharge::Precise(10));

        let events = monitor.observe_incremental_at(critical, true, now);

        assert!(events.is_empty());
    }

    #[test]
    fn defers_the_connected_event_for_an_unconfirmed_wired_classification() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        let connected = monitor.observe_incremental_at(wired.clone(), true, now);

        assert!(connected.is_empty());
        let confirmed =
            monitor.observe_pending_at(vec![wired.clone()], now + READING_CONFIRMATION_DELAY);
        assert_eq!(confirmed, vec![ControllerEvent::Connected(wired.clone())]);
        assert_eq!(
            confirmed[0]
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .body(),
            "Battery level is wired"
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
    fn settled_wireless_reading_is_used_in_the_connected_notification() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        let full = Controller::new(
            "one",
            BatteryReading::new(
                BatteryKind::Alkaline,
                BatteryCharge::Coarse(BatteryLevel::Full),
            ),
        );

        assert!(monitor.observe_current_at(vec![wired], now).is_empty());
        let events =
            monitor.observe_current_at(vec![full.clone()], now + Duration::from_millis(250));

        assert_eq!(events, vec![ControllerEvent::Connected(full)]);
        assert_eq!(
            events[0]
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .body(),
            "Battery level is ~100%"
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
            monitor.next_refresh_delay_at(old_deadline),
            Some(READING_CONFIRMATION_DELAY - Duration::from_millis(100))
        );

        let new_deadline = wired_seen + READING_CONFIRMATION_DELAY;
        assert!(
            monitor
                .observe_incremental_at(wired.clone(), true, new_deadline)
                .is_empty()
        );
        assert_eq!(monitor.next_refresh_delay_at(new_deadline), None);
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
            monitor.next_refresh_delay_at(old_deadline),
            Some(READING_CONFIRMATION_DELAY - Duration::from_millis(100))
        );

        let new_deadline = low_seen + READING_CONFIRMATION_DELAY;
        assert_eq!(
            monitor.observe_incremental_at(low.clone(), true, new_deadline),
            vec![ControllerEvent::Connected(low), critical_warning()]
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
            monitor
                .next_refresh_delay_at(now + READING_CONFIRMATION_DELAY + Duration::from_secs(1)),
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
            vec![
                ControllerEvent::Connected(critical.clone()),
                critical_warning()
            ]
        );
        assert!(
            monitor
                .observe_incremental_at(unknown, true, deadline)
                .is_empty()
        );
        assert_eq!(monitor.next_refresh_delay_at(deadline), None);
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
            critical.clone(),
            true,
            now + READING_CONFIRMATION_DELAY + Duration::from_secs(1),
        );

        assert_eq!(
            events,
            vec![
                ControllerEvent::Connected(critical.clone()),
                critical_warning()
            ]
        );
        assert_eq!(
            events[0]
                .notification(&ControllerNotificationPolicy::default())
                .unwrap()
                .body(),
            "Battery level is 10%"
        );
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
    fn snapshot_observations_confirm_a_persistent_wired_transition() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        let full = Controller::new(
            "one",
            BatteryReading::new(
                BatteryKind::Alkaline,
                BatteryCharge::Coarse(BatteryLevel::Full),
            ),
        );
        let wired = Controller::new(
            "one",
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        );
        monitor.observe_current_at(vec![full], now);
        let first_wired = now + Duration::from_secs(1);

        assert!(
            monitor
                .observe_current_at(vec![wired.clone()], first_wired)
                .is_empty()
        );
        let confirmed_at = first_wired + READING_CONFIRMATION_DELAY;
        assert!(
            monitor
                .observe_current_at(vec![wired.clone()], confirmed_at)
                .is_empty()
        );
        assert_eq!(monitor.next_refresh_delay_at(confirmed_at), None);
        assert_eq!(
            monitor.observe_current_at(Vec::new(), confirmed_at + Duration::from_secs(1)),
            vec![ControllerEvent::Disconnected(wired)]
        );
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
            monitor
                .observe_current_at(vec![coarse_empty.clone()], now + READING_CONFIRMATION_DELAY,),
            vec![
                ControllerEvent::Connected(coarse_empty),
                coarse_empty_warning()
            ]
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
            monitor.next_refresh_delay_at(first_seen + READING_CONFIRMATION_DELAY),
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

        assert_eq!(monitor.next_refresh_delay_at(final_refresh), None);
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
            monitor.observe_pending_at(vec![critical.clone()], confirmed_at),
            vec![ControllerEvent::Connected(critical), critical_warning()]
        );
    }

    #[test]
    fn repeated_refresh_errors_eventually_abandon_the_candidate() {
        let now = Instant::now();
        let mut monitor = ControllerMonitor::default();
        monitor.observe_incremental_at(controller("one", BatteryCharge::Precise(10)), true, now);

        for attempt in 1..=MAX_DEFERRED_REFRESHES {
            let refresh_at = now + READING_CONFIRMATION_DELAY * u32::from(attempt);
            assert!(monitor.defer_due_refreshes_at(refresh_at).is_empty());
        }

        let abandoned_at = now + READING_CONFIRMATION_DELAY * u32::from(MAX_DEFERRED_REFRESHES + 1);
        assert_eq!(
            monitor.defer_due_refreshes_at(abandoned_at),
            vec![ControllerEvent::Connected(controller(
                "one",
                BatteryCharge::Unknown
            ))]
        );
        assert_eq!(monitor.next_refresh_delay_at(abandoned_at), None);
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
            monitor.observe_pending_at(vec![critical.clone()], confirmed_at),
            vec![ControllerEvent::Connected(critical), critical_warning()]
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
            monitor.next_refresh_delay_at(now + Duration::from_secs(2)),
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
            critical.clone(),
            true,
            reconnected_at + READING_CONFIRMATION_DELAY,
        );

        assert_eq!(disconnected, vec![ControllerEvent::Disconnected(full)]);
        assert!(reconnected.is_empty());
        assert!(before_new_deadline.is_empty());
        assert_eq!(
            confirmed,
            vec![ControllerEvent::Connected(critical), critical_warning()]
        );
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

    fn wired_controller(id: &str) -> Controller {
        Controller::new(
            id,
            BatteryReading::new(BatteryKind::Wired, BatteryCharge::Unknown),
        )
    }

    fn critical_warning() -> ControllerEvent {
        ControllerEvent::BatteryWarning {
            warning: critical_warning_value(),
        }
    }

    fn critical_warning_value() -> BatteryWarning {
        BatteryWarning::precise(
            10,
            BatteryWarningLevel::new(
                "empty",
                Some(10),
                Some(BatteryLevel::Empty),
                true,
                true,
                None,
            ),
        )
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
