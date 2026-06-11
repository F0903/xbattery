use std::{collections::BTreeMap, time::Duration};

use serde::Deserialize;

use crate::{
    AppResult,
    controller::rumble::{ControllerRumbleConfig, RumbleJolt, RumblePattern, RumblePatternSet},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RumbleConfig {
    pub enabled: bool,
    pub gap_millis: u64,
    pub group_gap_millis: u64,
    pub jolts: BTreeMap<String, RumbleJoltConfig>,
    pub patterns: RumblePatternConfigSet,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RumbleJoltConfig {
    pub handle_strength_percent: u8,
    pub trigger_strength_percent: u8,
    pub handle_millis: u64,
    pub trigger_millis: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RumblePatternConfig {
    pub groups: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RumblePatternConfigSet {
    pub medium: Option<RumblePatternConfig>,
    pub low: Option<RumblePatternConfig>,
    pub empty: Option<RumblePatternConfig>,
}

impl RumbleConfig {
    pub fn controller_rumble_config(&self) -> AppResult<ControllerRumbleConfig> {
        let jolts = self.resolved_jolts()?;
        let medium = self.resolve_pattern(
            "medium",
            self.patterns.medium.as_ref(),
            &[&["quick", "quick"]],
            &jolts,
        )?;
        let low = self.resolve_pattern(
            "low",
            self.patterns.low.as_ref(),
            &[&["quick", "quick", "strong"]],
            &jolts,
        )?;
        let empty = self.resolve_pattern(
            "empty",
            self.patterns.empty.as_ref(),
            &[&["quick", "quick", "strong"], &["quick", "quick", "strong"]],
            &jolts,
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

    fn resolve_pattern(
        &self,
        stage: &str,
        pattern: Option<&RumblePatternConfig>,
        default_groups: &[&[&str]],
        jolts: &BTreeMap<String, RumbleJolt>,
    ) -> AppResult<RumblePattern> {
        let groups = match pattern {
            Some(pattern) => pattern
                .groups
                .iter()
                .map(|group| group.iter().map(String::as_str).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            None => default_groups.iter().map(|group| group.to_vec()).collect(),
        };

        if groups.is_empty() {
            return Err(format!("rumble.patterns.{stage}.groups must not be empty").into());
        }

        let mut resolved_groups = Vec::new();
        for (index, group) in groups.into_iter().enumerate() {
            if group.is_empty() {
                return Err(format!(
                    "rumble.patterns.{stage}.groups[{}] must not be empty",
                    index
                )
                .into());
            }

            let mut resolved_group = Vec::new();
            for jolt_name in group {
                let Some(jolt) = jolts.get(jolt_name).copied() else {
                    return Err(format!(
                        "rumble.patterns.{stage} references unknown jolt \"{jolt_name}\""
                    )
                    .into());
                };
                resolved_group.push(jolt);
            }
            resolved_groups.push(resolved_group);
        }

        Ok(RumblePattern::new(resolved_groups))
    }
}

impl RumbleJoltConfig {
    fn resolve(&self, name: &str) -> AppResult<RumbleJolt> {
        validate_percent_field(
            &format!("rumble.jolts.{name}.handle_strength_percent"),
            self.handle_strength_percent,
        )?;
        validate_percent_field(
            &format!("rumble.jolts.{name}.trigger_strength_percent"),
            self.trigger_strength_percent,
        )?;
        validate_duration(
            &format!("rumble.jolts.{name}.handle_millis"),
            self.handle_millis,
        )?;
        validate_duration(
            &format!("rumble.jolts.{name}.trigger_millis"),
            self.trigger_millis,
        )?;

        Ok(RumbleJolt::new(
            self.handle_strength_percent,
            self.trigger_strength_percent,
            Duration::from_millis(self.handle_millis),
            Duration::from_millis(self.trigger_millis),
        ))
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

fn validate_percent_field(field: &str, value: u8) -> AppResult<()> {
    if value > 100 {
        Err(format!("{field} must be between 0 and 100").into())
    } else {
        Ok(())
    }
}

fn validate_duration(field: &str, value: u64) -> AppResult<()> {
    if value == 0 {
        Err(format!("{field} must be greater than zero").into())
    } else {
        Ok(())
    }
}
