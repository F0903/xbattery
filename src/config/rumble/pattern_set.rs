use serde::Deserialize;

use super::RumblePatternConfig;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RumblePatternConfigSet {
    pub medium: Option<RumblePatternConfig>,
    pub low: Option<RumblePatternConfig>,
    pub empty: Option<RumblePatternConfig>,
}
