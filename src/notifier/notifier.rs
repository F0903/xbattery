use crate::AppResult;

use super::Notification;

pub trait Notifier {
    fn notify(&self, notification: &Notification) -> AppResult<()>;
}
