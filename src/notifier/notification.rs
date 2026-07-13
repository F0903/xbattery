#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationUrgency {
    Normal,
    High,
    Urgent,
}

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
