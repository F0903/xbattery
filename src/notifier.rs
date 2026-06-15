use std::sync::{Arc, RwLock};

use crate::{
    AppResult,
    toast::{Toast, ToastConfig},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notification {
    title: String,
    body: String,
    urgency: NotificationUrgency,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::with_urgency(title, body, NotificationUrgency::Normal)
    }

    pub fn with_urgency(
        title: impl Into<String>,
        body: impl Into<String>,
        urgency: NotificationUrgency,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            urgency,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn urgency(&self) -> NotificationUrgency {
        self.urgency
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationUrgency {
    Normal,
    High,
    Urgent,
}

pub trait Notifier {
    fn notify(&self, notification: &Notification) -> AppResult<()>;
}

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
