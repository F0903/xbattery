use std::sync::{Arc, RwLock};

use crate::{
    AppResult,
    toast::{Toast, ToastConfig},
};

use super::{Notification, Notifier};

#[derive(Clone, Debug)]
pub struct ToastNotifier {
    config: Arc<RwLock<ToastConfig>>,
}

impl ToastNotifier {
    pub fn new(config: ToastConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn set_config(&self, config: ToastConfig) -> AppResult<()> {
        *self
            .config
            .write()
            .map_err(|_| "toast config lock is poisoned")? = config;
        Ok(())
    }

    fn config(&self) -> AppResult<ToastConfig> {
        Ok(self
            .config
            .read()
            .map_err(|_| "toast config lock is poisoned")?
            .clone())
    }
}

impl Default for ToastNotifier {
    fn default() -> Self {
        Self::new(ToastConfig::default())
    }
}

impl Notifier for ToastNotifier {
    fn notify(&self, notification: &Notification) -> AppResult<()> {
        Toast::with_config_and_urgency(
            self.config()?,
            notification.title(),
            notification.body(),
            notification.urgency(),
        )
        .send()
    }
}
