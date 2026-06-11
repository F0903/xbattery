use windows::Gaming::Input::RawGameController;

use crate::{
    AppResult,
    battery::{BatteryCharge, BatteryKind, BatteryReading},
};

#[derive(Debug)]
pub struct RawControllerReport {
    pub id: String,
    pub display_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub is_wireless: bool,
    pub remaining_mwh: Option<i32>,
    pub full_charge_mwh: Option<i32>,
    pub percent: Option<u8>,
}

impl RawControllerReport {
    pub fn battery(&self) -> BatteryReading {
        let kind = if self.is_wireless {
            BatteryKind::Unknown
        } else {
            BatteryKind::Wired
        };

        BatteryReading::new(
            kind,
            self.percent
                .map(BatteryCharge::Precise)
                .unwrap_or(BatteryCharge::Unknown),
        )
    }

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

pub fn raw_controller_reports() -> AppResult<Vec<RawControllerReport>> {
    let controllers = RawGameController::RawGameControllers()?;
    let mut reports = Vec::with_capacity(controllers.Size()? as usize);

    for index in 0..controllers.Size()? {
        let controller = controllers.GetAt(index)?;
        let id = controller.NonRoamableId()?.to_string();
        let display_name = controller.DisplayName()?.to_string();
        let report = controller.TryGetBatteryReport()?;
        let remaining_mwh = report
            .RemainingCapacityInMilliwattHours()
            .ok()
            .and_then(|value| value.Value().ok());
        let full_charge_mwh = report
            .FullChargeCapacityInMilliwattHours()
            .ok()
            .and_then(|value| value.Value().ok());
        let percent = match (remaining_mwh, full_charge_mwh) {
            (Some(remaining), Some(full_charge)) if full_charge > 0 => {
                let value = ((remaining as f64 / full_charge as f64) * 100.0).round();
                Some(value.clamp(0.0, 100.0) as u8)
            }
            _ => None,
        };

        reports.push(RawControllerReport {
            id,
            display_name,
            vendor_id: controller.HardwareVendorId()?,
            product_id: controller.HardwareProductId()?,
            is_wireless: controller.IsWireless()?,
            remaining_mwh,
            full_charge_mwh,
            percent,
        });
    }

    Ok(reports)
}
