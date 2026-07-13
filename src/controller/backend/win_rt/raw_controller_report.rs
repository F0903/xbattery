#[derive(Debug)]
pub struct RawControllerReport {
    pub display_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub is_wireless: bool,
    pub remaining_mwh: Option<i32>,
    pub full_charge_mwh: Option<i32>,
    pub percent: Option<u8>,
}

impl RawControllerReport {
    pub fn description(&self) -> String {
        let percent = self
            .percent
            .map(|value| format!("{}%", value))
            .unwrap_or_else(|| "unknown percent".to_string());

        format!(
            "{} (vid {:04x}, pid {:04x}, {}, {})",
            self.display_name,
            self.vendor_id,
            self.product_id,
            if self.is_wireless {
                "wireless"
            } else {
                "wired"
            },
            percent
        )
    }
}
