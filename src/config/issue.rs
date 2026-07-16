use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConfigIssue {
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

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConfigIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}
