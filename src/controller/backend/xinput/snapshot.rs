use crate::controller::battery::BatteryReading;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerSnapshot {
    pub slot: u32,
    pub packet_number: u32,
    pub battery: BatteryReading,
}

impl ControllerSnapshot {
    pub fn name(self) -> String {
        format!("Controller {}", self.slot + 1)
    }
}
