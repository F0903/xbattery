use crate::notifier::{Notification, NotificationUrgency, Notifier, ToastNotifier};

pub(super) fn notify_update_available(notifier: &ToastNotifier, latest_version: &str) {
    let notification = Notification::new(
        "xbattery Update Available",
        format!("Version {latest_version} is available. Run xbattery.exe update to install it."),
    );

    let _ = notifier.notify(&notification);
}

pub(super) fn notify_auto_update_started(notifier: &ToastNotifier, latest_version: &str) {
    let notification = Notification::with_urgency(
        "xbattery Update Started",
        format!("Version {latest_version} is available. xbattery will restart after updating."),
        NotificationUrgency::High,
    );

    let _ = notifier.notify(&notification);
}
