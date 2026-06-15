use std::collections::BTreeMap;

use crate::{
    AppResult,
    controller::rumble::{RumbleJolt, RumblePattern},
};

use super::RumblePatternConfig;

pub(super) struct RumblePatternResolver<'a> {
    jolts: &'a BTreeMap<String, RumbleJolt>,
}

impl<'a> RumblePatternResolver<'a> {
    pub(super) fn new(jolts: &'a BTreeMap<String, RumbleJolt>) -> Self {
        Self { jolts }
    }

    pub(super) fn resolve(
        &self,
        stage: &str,
        pattern: Option<&RumblePatternConfig>,
        default_groups: &[&[&str]],
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
                let Some(jolt) = self.jolts.get(jolt_name).copied() else {
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
