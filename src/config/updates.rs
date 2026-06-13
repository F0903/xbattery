use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpdatesConfig {
    pub repo_owner: String,
    pub repo_name: String,
    pub asset_identifier: String,
    pub bin_path_in_archive: String,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            repo_owner: "F0903".to_string(),
            repo_name: "xbattery".to_string(),
            asset_identifier: "xbattery".to_string(),
            bin_path_in_archive: "xbattery.exe".to_string(),
        }
    }
}
