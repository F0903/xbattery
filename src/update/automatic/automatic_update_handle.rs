use std::sync::{Arc, RwLock};

use crate::{AppResult, config::UpdatesConfig};

#[derive(Clone, Debug)]
pub struct AutomaticUpdateHandle {
    config: Option<Arc<RwLock<UpdatesConfig>>>,
}

impl AutomaticUpdateHandle {
    pub(super) fn disabled() -> Self {
        Self { config: None }
    }

    pub(super) fn enabled(config: Arc<RwLock<UpdatesConfig>>) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub fn update_config(&self, config: UpdatesConfig) -> AppResult<()> {
        if let Some(current) = &self.config {
            *current
                .write()
                .map_err(|_| "automatic update config lock is poisoned")? = config;
        }

        Ok(())
    }
}
