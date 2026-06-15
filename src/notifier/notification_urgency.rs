#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationUrgency {
    Normal,
    High,
    Urgent,
}

impl NotificationUrgency {
    pub(crate) fn uses_high_priority(self) -> bool {
        matches!(self, Self::High | Self::Urgent)
    }

    pub(crate) fn toast_scenario(self) -> Option<&'static str> {
        match self {
            Self::Urgent => Some("urgent"),
            Self::Normal | Self::High => None,
        }
    }
}
