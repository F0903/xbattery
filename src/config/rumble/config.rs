use std::{collections::BTreeMap, time::Duration};

use serde::Deserialize;

use crate::{
    AppResult,
    controller::rumble::{ControllerRumbleConfig, RumbleJolt, RumblePatternSet},
};

use super::{RumbleJoltConfig, RumblePatternConfigSet, pattern_resolver::RumblePatternResolver};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RumbleConfig {
    pub enabled: bool,
    pub gap_millis: u64,
    pub group_gap_millis: u64,
    pub jolts: BTreeMap<String, RumbleJoltConfig>,
    pub patterns: RumblePatternConfigSet,
}

impl RumbleConfig {
    pub fn controller_rumble_config(&self) -> AppResult<ControllerRumbleConfig> {
        let jolts = self.resolved_jolts()?;
        let resolver = RumblePatternResolver::new(&jolts);
        let medium = resolver.resolve(
            "medium",
            self.patterns.medium.as_ref(),
            &[&["quick", "quick"]],
        )?;
        let low = resolver.resolve(
            "low",
            self.patterns.low.as_ref(),
            &[&["quick", "quick", "strong"]],
        )?;
        let empty = resolver.resolve(
            "empty",
            self.patterns.empty.as_ref(),
            &[&["quick", "quick", "strong"], &["quick", "quick", "strong"]],
        )?;

        Ok(ControllerRumbleConfig::custom(
            self.enabled,
            Duration::from_millis(self.gap_millis),
            Duration::from_millis(self.group_gap_millis),
            RumblePatternSet::new(medium, low, empty),
        ))
    }

    fn resolved_jolts(&self) -> AppResult<BTreeMap<String, RumbleJolt>> {
        let mut jolts = BTreeMap::new();

        jolts.insert(
            "quick".to_owned(),
            RumbleJolt::new(
                100,
                75,
                Duration::from_millis(35),
                Duration::from_millis(50),
            ),
        );
        jolts.insert(
            "strong".to_owned(),
            RumbleJolt::new(
                100,
                100,
                Duration::from_millis(75),
                Duration::from_millis(100),
            ),
        );

        for (name, config) in &self.jolts {
            jolts.insert(name.clone(), config.resolve(name)?);
        }

        Ok(jolts)
    }
}

impl Default for RumbleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            gap_millis: 45,
            group_gap_millis: 200,
            jolts: BTreeMap::new(),
            patterns: RumblePatternConfigSet::default(),
        }
    }
}
