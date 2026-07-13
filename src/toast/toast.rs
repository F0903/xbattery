use windows::{
    UI::Notifications::{
        ToastNotification, ToastNotificationManager, ToastNotificationPriority, ToastTemplateType,
    },
    core::HSTRING,
};

use crate::{AppResult, notifier::NotificationUrgency};

use super::ToastConfig;

pub struct Toast {
    config: ToastConfig,
    title: String,
    body: String,
    urgency: NotificationUrgency,
}

impl Toast {
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
