use windows::{
    UI::Notifications::{
        ToastNotification, ToastNotificationManager, ToastNotificationPriority, ToastTemplateType,
    },
    core::HSTRING,
};

use crate::{AppResult, notifier::NotificationUrgency};

const DEFAULT_APP_ID: &str = "xbattery";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastConfig {
    app_id: String,
}

impl ToastConfig {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
        }
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self::new(DEFAULT_APP_ID)
    }
}

pub struct Toast {
    config: ToastConfig,
    title: String,
    body: String,
    urgency: NotificationUrgency,
}

impl Toast {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::with_config_and_urgency(
            ToastConfig::default(),
            title,
            body,
            NotificationUrgency::Normal,
        )
    }

    pub fn with_urgency(
        title: impl Into<String>,
        body: impl Into<String>,
        urgency: NotificationUrgency,
    ) -> Self {
        Self::with_config_and_urgency(ToastConfig::default(), title, body, urgency)
    }

    pub fn with_config(
        config: ToastConfig,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self::with_config_and_urgency(config, title, body, NotificationUrgency::Normal)
    }

    pub fn with_config_and_urgency(
        config: ToastConfig,
        title: impl Into<String>,
        body: impl Into<String>,
        urgency: NotificationUrgency,
    ) -> Self {
        Self {
            config,
            title: title.into(),
            body: body.into(),
            urgency,
        }
    }

    pub fn send(&self) -> AppResult<()> {
        let toast = self.create_notification()?;
        toast.SetExpiresOnReboot(true)?;
        if self.urgency.uses_high_priority() {
            let _ = toast.SetPriority(ToastNotificationPriority::High);
        }

        let app_id = HSTRING::from(self.config.app_id());
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id)?;
        notifier.Show(&toast)?;

        Ok(())
    }

    fn create_notification(&self) -> AppResult<ToastNotification> {
        let toast_xml =
            ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)?;
        if let Some(scenario) = self.urgency.toast_scenario() {
            toast_xml
                .DocumentElement()?
                .SetAttribute(&HSTRING::from("scenario"), &HSTRING::from(scenario))?;
        }

        let text_nodes = toast_xml.GetElementsByTagName(&HSTRING::from("text"))?;
        let values = [&self.title, &self.body];

        for index in 0..text_nodes.Size()?.min(values.len() as u32) {
            let element = text_nodes.GetAt(index)?;
            let node = toast_xml.CreateTextNode(&HSTRING::from(values[index as usize]))?;
            element.AppendChild(&node)?;
        }

        Ok(ToastNotification::CreateToastNotification(&toast_xml)?)
    }
}

impl NotificationUrgency {
    fn uses_high_priority(self) -> bool {
        matches!(self, Self::High | Self::Urgent)
    }

    fn toast_scenario(self) -> Option<&'static str> {
        match self {
            Self::Urgent => Some("urgent"),
            Self::Normal | Self::High => None,
        }
    }
}
