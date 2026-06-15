use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RumblePatternConfig {
    pub groups: Vec<Vec<String>>,
}
