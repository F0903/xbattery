use std::time::Duration;

use serde::Deserialize;

use crate::{AppResult, controller::rumble::RumbleJolt};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RumbleJoltConfig {
    pub handle_strength_percent: u8,
    pub trigger_strength_percent: u8,
    pub handle_millis: u64,
    pub trigger_millis: u64,
}

impl RumbleJoltConfig {
    pub(super) fn resolve(&self, name: &str) -> AppResult<RumbleJolt> {
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
