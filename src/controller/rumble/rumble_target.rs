use crate::controller::{Controller, ControllerSource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RumbleTarget {
    SingleController,
    XInputSlot(u32),
}

impl RumbleTarget {
    pub fn for_controller(controller: &Controller) -> Self {
        if controller.source() == ControllerSource::XInput
            && let Some(slot) = parse_xinput_slot(controller.id())
        {
            return Self::XInputSlot(slot);
        }

        Self::SingleController
    }

    pub fn xinput_slot(self) -> Option<u32> {
        match self {
            Self::XInputSlot(slot) => Some(slot),
            Self::SingleController => None,
        }
    }
}

fn parse_xinput_slot(id: &str) -> Option<u32> {
    id.strip_prefix("xinput:")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use crate::controller::{
        Controller, ControllerSource,
        battery::{BatteryCharge, BatteryKind, BatteryReading},
    };

    use super::RumbleTarget;

    #[test]
    fn targets_exact_xinput_slot_for_xinput_controller() {
        let controller = Controller::new(
            "xinput:2",
            "Controller",
            ControllerSource::XInput,
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown),
        );

        assert_eq!(
            RumbleTarget::for_controller(&controller),
            RumbleTarget::XInputSlot(2)
        );
    }

    #[test]
    fn targets_single_controller_for_non_xinput_controller() {
        let controller = Controller::new(
            "gameinput:123",
            "Controller",
            ControllerSource::GameInput,
            BatteryReading::new(BatteryKind::Unknown, BatteryCharge::Unknown),
        );

        assert_eq!(
            RumbleTarget::for_controller(&controller),
            RumbleTarget::SingleController
        );
    }
}
