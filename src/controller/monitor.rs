use std::collections::HashMap;

use crate::controller::{
    battery::{BatteryWarning, BatteryWarningPolicy},
    event::ControllerEvent,
};

use super::Controller;

#[derive(Clone, Debug, Default)]
pub struct ControllerMonitor {
    previous: Vec<Controller>,
    warning_policy: BatteryWarningPolicy,
}

impl ControllerMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_warning_policy(warning_policy: BatteryWarningPolicy) -> Self {
        Self {
            previous: Vec::new(),
            warning_policy,
        }
    }

    pub fn set_warning_policy(&mut self, warning_policy: BatteryWarningPolicy) {
        self.warning_policy = warning_policy;
    }

    pub fn observe_current(&mut self, current: Vec<Controller>) -> Vec<ControllerEvent> {
        let events = self.collect_events(&current);
        self.previous = current;
        events
    }

    pub fn observe_incremental(
        &mut self,
        controller: Controller,
        is_connected: bool,
    ) -> Vec<ControllerEvent> {
        let mut events = Vec::new();

        if is_connected {
            match self
                .previous
                .iter()
                .position(|previous| previous.id() == controller.id())
            {
                Some(index) => {
                    if let Some(warning) = self.warning_between(&self.previous[index], &controller)
                    {
                        events.push(ControllerEvent::BatteryWarning {
                            current: controller.clone(),
                            warning,
                        });
                    }

                    self.previous[index] = controller;
                }
                None => {
                    events.push(ControllerEvent::Connected(controller.clone()));
                    self.previous.push(controller);
                }
            }
        } else if let Some(index) = self
            .previous
            .iter()
            .position(|previous| previous.id() == controller.id())
        {
            events.push(ControllerEvent::Disconnected(self.previous.remove(index)));
        }

        events
    }

    fn collect_events(&self, current: &[Controller]) -> Vec<ControllerEvent> {
        let mut events = Vec::new();
        let previous_by_id = self
            .previous
            .iter()
            .map(|controller| (controller.id(), controller))
            .collect::<HashMap<_, _>>();
        let current_by_id = current
            .iter()
            .map(|controller| (controller.id(), controller))
            .collect::<HashMap<_, _>>();

        for controller in current {
            match previous_by_id.get(controller.id()) {
                None => events.push(ControllerEvent::Connected(controller.clone())),
                Some(previous) => {
                    if let Some(warning) = self.warning_between(previous, controller) {
                        events.push(ControllerEvent::BatteryWarning {
                            current: controller.clone(),
                            warning,
                        });
                    }
                }
            }
        }

        for controller in &self.previous {
            if !current_by_id.contains_key(controller.id()) {
                events.push(ControllerEvent::Disconnected(controller.clone()));
            }
        }

        events
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

#[cfg(test)]
mod tests {
    use crate::controller::{
        Controller, ControllerSource,
        battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading},
        event::ControllerEvent,
    };

    use super::ControllerMonitor;

    #[test]
    fn emits_connected_event_for_new_controller() {
        let mut monitor = ControllerMonitor::new();
        let controller = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));

        let events = monitor.observe_current(vec![controller.clone()]);

        assert_eq!(events, vec![ControllerEvent::Connected(controller)]);
    }

    #[test]
    fn emits_disconnected_event_for_missing_controller() {
        let mut monitor = ControllerMonitor::new();
        let controller = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        monitor.observe_current(vec![controller.clone()]);

        let events = monitor.observe_current(Vec::new());

        assert_eq!(events, vec![ControllerEvent::Disconnected(controller)]);
    }

    #[test]
    fn emits_battery_warning_for_decreasing_coarse_level() {
        let mut monitor = ControllerMonitor::new();
        let full = controller("one", BatteryCharge::Coarse(BatteryLevel::Full));
        let medium = controller("one", BatteryCharge::Coarse(BatteryLevel::Medium));
        monitor.observe_current(vec![full]);

        let events = monitor.observe_current(vec![medium.clone()]);

        assert_eq!(
            events,
            vec![ControllerEvent::BatteryWarning {
                current: medium,
                warning: crate::controller::battery::BatteryWarning::Coarse(BatteryLevel::Medium),
            }]
        );
    }

    fn controller(id: &str, charge: BatteryCharge) -> Controller {
        Controller::new(
            id,
            "Controller",
            ControllerSource::XInput,
            BatteryReading::new(BatteryKind::Unknown, charge),
        )
    }
}
