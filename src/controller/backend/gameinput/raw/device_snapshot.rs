use crate::battery::BatteryReading;

use super::{
    GameInputBatteryState, battery_state::battery_status_description,
    constants::GAMEINPUT_DEVICE_CONNECTED,
};

#[derive(Clone, Debug)]
pub struct GameInputDeviceSnapshot {
    pub id: String,
    pub name: String,
    pub timestamp: u64,
    pub current_status: i32,
    pub previous_status: i32,
    pub battery: BatteryReading,
    pub raw_battery: GameInputBatteryState,
}

impl GameInputDeviceSnapshot {
    pub fn is_connected(&self) -> bool {
        self.current_status & GAMEINPUT_DEVICE_CONNECTED != 0
    }

    pub fn current_status_description(&self) -> String {
        status_description(self.current_status)
    }

    pub fn previous_status_description(&self) -> String {
        status_description(self.previous_status)
    }

    pub fn battery_status_description(&self) -> &'static str {
        battery_status_description(self.raw_battery.status)
    }
}

fn status_description(status: i32) -> String {
    let flags = [
        (0x0000_0001, "connected"),
        (0x0000_0002, "input-enabled"),
        (0x0000_0004, "output-enabled"),
        (0x0000_0008, "raw-io-enabled"),
        (0x0000_0010, "audio-capture"),
        (0x0000_0020, "audio-render"),
    ];
    let mut parts = flags
        .iter()
        .filter_map(|(flag, label)| (status & flag != 0).then_some(*label))
        .collect::<Vec<_>>();

    if parts.is_empty() {
        parts.push("none");
    }

    parts.join(", ")
}
