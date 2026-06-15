#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RumbleBackend {
    GameInput,
    WinRT,
    XInput(u32),
}

impl RumbleBackend {
    pub fn description(self) -> String {
        match self {
            Self::GameInput => "GameInput".to_string(),
            Self::WinRT => "WinRT Gamepad".to_string(),
            Self::XInput(slot) => format!("XInput slot {}", slot + 1),
        }
    }
}
