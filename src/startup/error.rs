use std::{error::Error, fmt};

#[derive(Debug)]
pub struct StartupAccessDenied {
    operation: &'static str,
    details: String,
}

impl StartupAccessDenied {
    pub(super) fn new(operation: &'static str, details: impl Into<String>) -> Self {
        Self {
            operation,
            details: details.into(),
        }
    }
}

impl fmt::Display for StartupAccessDenied {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Windows denied access while trying to {}. Run xbattery as administrator, or start it again and approve the UAC prompt.",
            self.operation
        )?;

        if !self.details.trim().is_empty() {
            write!(formatter, "\n\n{}", self.details.trim())?;
        }

        Ok(())
    }
}

impl Error for StartupAccessDenied {}

pub fn is_startup_access_denied(error: &(dyn Error + 'static)) -> bool {
    error.downcast_ref::<StartupAccessDenied>().is_some()
}
