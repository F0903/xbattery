use crate::controller::battery::BatteryReading;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Controller {
    id: String,
    battery: BatteryReading,
}

impl Controller {
    pub fn new(id: impl Into<String>, battery: BatteryReading) -> Self {
        Self {
            id: id.into(),
            battery,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn battery(&self) -> BatteryReading {
        self.battery
    }

    pub fn with_battery(mut self, battery: BatteryReading) -> Self {
        self.battery = battery;
        self
    }
}
