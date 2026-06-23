use std::collections::BTreeMap;

use serde::Deserialize;

use crate::controller::battery::{BatteryLevel, BatteryWarningLevel};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryConfig {
    pub levels: Option<BTreeMap<String, BatteryLevelConfig>>,
    pub precise_warning_thresholds: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryLevelConfig {
    pub threshold_percent: Option<u8>,
    pub coarse_level: Option<BatteryLevel>,
    pub notify: Option<bool>,
    pub urgent: bool,
}

impl BatteryConfig {
    pub fn warning_levels(
        &self,
        legacy_urgent_threshold_percent: Option<u8>,
    ) -> Vec<BatteryWarningLevel> {
        if let Some(thresholds) = &self.precise_warning_thresholds {
            return legacy_warning_levels(
                thresholds,
                legacy_urgent_threshold_percent.unwrap_or(10),
            );
        }

        match &self.levels {
            Some(levels) => levels
                .iter()
                .map(|(name, config)| config.warning_level(name))
                .collect(),
            None => BatteryWarningLevel::default_levels_with_urgent_threshold(
                legacy_urgent_threshold_percent.unwrap_or(10),
            ),
        }
    }
}

impl BatteryLevelConfig {
    pub fn warning_level(&self, name: &str) -> BatteryWarningLevel {
        BatteryWarningLevel::with_notify(
            name,
            self.threshold_percent,
            self.coarse_level,
            self.notify.unwrap_or(true),
            self.urgent,
        )
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            levels: None,
            precise_warning_thresholds: None,
        }
    }
}

fn legacy_warning_levels(
    thresholds: &[u8],
    urgent_threshold_percent: u8,
) -> Vec<BatteryWarningLevel> {
    let mut thresholds = thresholds.to_vec();
    thresholds.sort_unstable_by(|left, right| right.cmp(left));
    thresholds.dedup();

    let mut levels = thresholds
        .into_iter()
        .map(|threshold| {
            BatteryWarningLevel::new(
                format!("{threshold}%"),
                Some(threshold),
                None,
                threshold <= urgent_threshold_percent,
            )
        })
        .collect::<Vec<_>>();

    levels.extend(
        BatteryWarningLevel::default_levels_with_urgent_threshold(urgent_threshold_percent)
            .into_iter()
            .map(|level| {
                BatteryWarningLevel::with_notify(
                    level.name(),
                    None,
                    level.coarse_level(),
                    level.notify(),
                    level.urgent(),
                )
            }),
    );

    levels
}
