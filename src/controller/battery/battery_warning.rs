use crate::controller::battery::BatteryLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryWarning {
    Precise(u8),
    Coarse(BatteryLevel),
}
