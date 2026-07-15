use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigIssue {
    path: PathBuf,
    message: String,
}

impl ConfigIssue {
    pub(crate) fn new(path: PathBuf, error: impl fmt::Display) -> Self {
        Self {
            path,
            message: error.to_string(),
        }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConfigIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}
