use crate::controller::battery::BatteryReading;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XInputDiagnosticReport {
    pub slot: u32,
    pub packet_number: Option<u32>,
    pub battery: Option<BatteryReading>,
}

impl XInputDiagnosticReport {
    pub fn is_connected(self) -> bool {
        self.packet_number.is_some()
    }
}
