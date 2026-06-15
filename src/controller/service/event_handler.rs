use crate::{
    AppResult,
    controller::{
        backend::{
            BackendEvent, ControllerBattery, ControllerEventInput, ControllerInput,
            ControllerRumbler,
        },
        battery_source::{attach_battery_readings, attach_single_battery_reading},
        event::ControllerEvent,
    },
    notifier::Notifier,
};

use super::ControllerService;

impl<N, I, B, R> ControllerService<N, I, B, R>
where
    N: Notifier,
    I: ControllerInput + ControllerEventInput,
    B: ControllerBattery,
    R: ControllerRumbler + Clone + Send + 'static,
{
    pub(super) fn poll_and_notify(&mut self) -> AppResult<()> {
        let current = self.input.poll_controllers()?;
        let current = attach_battery_readings(current, &self.battery);
        let events = self.monitor.observe_current(current);
        self.notify_events(events)
    }

    pub(super) fn process_backend_event(&mut self, event: BackendEvent) -> AppResult<()> {
        let (controller, is_connected) = self.input.controller_from_event(event);
        let controller = attach_single_battery_reading(controller, &self.battery);
        let events = self.monitor.observe_incremental(controller, is_connected);

        self.notify_events(events)
    }

    fn notify_events(&self, events: Vec<ControllerEvent>) -> AppResult<()> {
        for event in events {
            self.rumbler.rumble_for_event(&event);

            if let Some(notification) = event.notification(self.config.notification_policy()) {
                self.notifier.notify(&notification)?;
            }
        }

        Ok(())
    }
}
