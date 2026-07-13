use std::sync::{Arc, RwLock};

use windows::{
    UI::Notifications::{
        ToastNotification, ToastNotificationManager, ToastNotificationPriority, ToastTemplateType,
    },
    core::HSTRING,
};

use crate::AppResult;

use super::{Notification, NotificationUrgency, Notifier};

#[derive(Clone, Debug)]
pub struct ToastNotifier {
    app_id: Arc<RwLock<String>>,
}

impl ToastNotifier {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: Arc::new(RwLock::new(app_id.into())),
        }
    }

    pub fn set_app_id(&self, app_id: impl Into<String>) -> AppResult<()> {
        *self
            .app_id
            .write()
            .map_err(|_| "toast app ID lock is poisoned")? = app_id.into();
        Ok(())
    }

    fn current_app_id(&self) -> AppResult<String> {
        Ok(self
            .app_id
            .read()
            .map_err(|_| "toast app ID lock is poisoned")?
            .clone())
    }

    fn create_notification(notification: &Notification) -> AppResult<ToastNotification> {
        let toast_xml =
            ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)?;
        if notification.urgency() == NotificationUrgency::Urgent {
            toast_xml
                .DocumentElement()?
                .SetAttribute(&HSTRING::from("scenario"), &HSTRING::from("urgent"))?;
        }

        let text_nodes = toast_xml.GetElementsByTagName(&HSTRING::from("text"))?;
        let values = [notification.title(), notification.body()];

        for index in 0..text_nodes.Size()?.min(values.len() as u32) {
            let element = text_nodes.GetAt(index)?;
            let node = toast_xml.CreateTextNode(&HSTRING::from(values[index as usize]))?;
            element.AppendChild(&node)?;
        }

        Ok(ToastNotification::CreateToastNotification(&toast_xml)?)
    }
}

impl Notifier for ToastNotifier {
    fn notify(&self, notification: &Notification) -> AppResult<()> {
        let toast = Self::create_notification(notification)?;
        toast.SetExpiresOnReboot(true)?;
        if matches!(
            notification.urgency(),
            NotificationUrgency::High | NotificationUrgency::Urgent
        ) {
            let _ = toast.SetPriority(ToastNotificationPriority::High);
        }

        let app_id = HSTRING::from(self.current_app_id()?);
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id)?;
        notifier.Show(&toast)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ToastNotifier;

    #[test]
    fn updates_the_app_id_shared_by_clones() {
        let notifier = ToastNotifier::new("initial");
        let clone = notifier.clone();

        clone.set_app_id("updated").unwrap();

        assert_eq!(notifier.current_app_id().unwrap(), "updated");
    }
}
