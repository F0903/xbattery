use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryConfig {
    pub precise_warning_thresholds: Vec<u8>,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            precise_warning_thresholds: vec![50, 25, 10],
        }
    }
}
