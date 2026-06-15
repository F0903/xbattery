use super::{BatteryCharge, BatteryKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryReading {
    pub kind: BatteryKind,
    pub charge: BatteryCharge,
}

impl BatteryReading {
    pub fn new(kind: BatteryKind, charge: BatteryCharge) -> Self {
        Self { kind, charge }
    }

    pub fn description(self) -> String {
        match (self.kind, self.charge) {
            (BatteryKind::Wired, _) => "wired".to_string(),
            (_, BatteryCharge::Unknown) => self.kind.to_string(),
            (_, charge) => charge.description(),
        }
    }
}
