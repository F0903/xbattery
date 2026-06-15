use crate::controller::battery::BatteryReading;

use super::{ControllerSource, backend::BackendKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Controller {
    id: String,
    name: String,
    source: ControllerSource,
    battery_source: BackendKind,
    battery: BatteryReading,
}

impl Controller {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        source: ControllerSource,
        battery: BatteryReading,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            battery_source: source.backend_kind(),
            source,
            battery,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> ControllerSource {
        self.source
    }

    pub fn battery(&self) -> BatteryReading {
        self.battery
    }

    pub fn battery_source(&self) -> BackendKind {
        self.battery_source
    }

    pub fn with_battery(mut self, source: BackendKind, battery: BatteryReading) -> Self {
        self.battery_source = source;
        self.battery = battery;
        self
    }
}
