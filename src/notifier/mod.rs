mod notification;
mod toast_notifier;

use crate::AppResult;

pub use notification::{Notification, NotificationUrgency};
pub use toast_notifier::ToastNotifier;

pub trait Notifier {
    fn notify(&self, notification: &Notification) -> AppResult<()>;
}
