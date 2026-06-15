#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckUpdateReport {
    current_version: String,
    latest_version: Option<String>,
}

impl CheckUpdateReport {
    pub(in crate::update) fn available(
        current_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: Some(latest_version.into()),
        }
    }

    pub(in crate::update) fn up_to_date(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: None,
        }
    }

    pub fn summary(&self) -> String {
        match &self.latest_version {
            Some(latest_version) => format!(
                "Update available: {} -> {}",
                self.current_version, latest_version
            ),
            None => format!("xbattery is up to date: {}", self.current_version),
        }
    }

    pub fn latest_version(&self) -> Option<&str> {
        self.latest_version.as_deref()
    }
}
