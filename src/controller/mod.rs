use crate::battery::{BatteryCharge, BatteryKind, BatteryReading};

pub mod event;
pub mod factory;
pub mod monitor;
pub mod poller;
pub mod rumble;
pub mod service;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Controller {
    id: String,
    name: String,
    source: ControllerSource,
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

    pub fn needs_battery_fallback(&self) -> bool {
        matches!(self.battery.charge, BatteryCharge::Unknown)
            || matches!(self.battery.kind, BatteryKind::Wired)
    }

    pub fn with_battery(mut self, source: ControllerSource, battery: BatteryReading) -> Self {
        self.source = source;
        self.battery = battery;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControllerSource {
    GameInput,
    GameInputWithXInputBattery,
    XInput,
    Winrt,
}

impl ControllerSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::GameInput => "GameInput",
            Self::GameInputWithXInputBattery => "GameInput + XInput battery",
            Self::XInput => "XInput",
            Self::Winrt => "Windows.Gaming.Input",
        }
    }
}
