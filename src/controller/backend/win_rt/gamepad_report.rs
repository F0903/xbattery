#[derive(Debug)]
pub struct GamepadReport {
    pub index: u32,
    pub is_wireless: bool,
    pub remaining_mwh: Option<i32>,
    pub full_charge_mwh: Option<i32>,
    pub percent: Option<u8>,
}

impl GamepadReport {
    pub fn description(&self) -> String {
        let percent = self
            .percent
            .map(|value| format!("{}%", value))
            .unwrap_or_else(|| "unknown percent".to_string());

        format!(
            "Gamepad {} ({}, {})",
            self.index + 1,
            if self.is_wireless {
                "wireless"
            } else {
                "wired"
            },
            percent
        )
    }
}
