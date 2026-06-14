use crate::monitor_control::MonitorStopResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckUpdateReport {
    current_version: String,
    latest_version: Option<String>,
}

impl CheckUpdateReport {
    pub(super) fn available(
        current_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: Some(latest_version.into()),
        }
    }

    pub(super) fn up_to_date(current_version: impl Into<String>) -> Self {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateReport {
    current_version: String,
    latest_version: Option<String>,
    kind: UpdateReportKind,
    monitor_stop_result: MonitorStopResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateReportKind {
    UpToDate,
    DryRun,
    Updated,
}

impl UpdateReport {
    pub(super) fn up_to_date(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: None,
            kind: UpdateReportKind::UpToDate,
            monitor_stop_result: MonitorStopResult::NotRunning,
        }
    }

    pub(super) fn dry_run(
        current_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: Some(latest_version.into()),
            kind: UpdateReportKind::DryRun,
            monitor_stop_result: MonitorStopResult::NotRunning,
        }
    }

    pub(super) fn updated(
        previous_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: previous_version.into(),
            latest_version: Some(latest_version.into()),
            kind: UpdateReportKind::Updated,
            monitor_stop_result: MonitorStopResult::NotRunning,
        }
    }

    pub(super) fn with_monitor_stop_result(mut self, result: MonitorStopResult) -> Self {
        self.monitor_stop_result = result;
        self
    }

    pub fn summary(&self) -> String {
        let update_line = match (self.kind, &self.latest_version) {
            (UpdateReportKind::UpToDate, _) => {
                format!("xbattery is up to date: {}", self.current_version)
            }
            (UpdateReportKind::DryRun, Some(latest_version)) => format!(
                "Update available: {} -> {}. Dry run; no files were changed.",
                self.current_version, latest_version
            ),
            (UpdateReportKind::Updated, Some(latest_version)) => {
                format!(
                    "Updated xbattery: {} -> {}",
                    self.current_version, latest_version
                )
            }
            (_, None) => "No update was applied.".to_string(),
        };

        match self.monitor_stop_result {
            MonitorStopResult::NotRunning => update_line,
            MonitorStopResult::Stopped => format!("{update_line}\nRestarted xbattery monitor."),
            MonitorStopResult::TimedOut => format!("{update_line}\nMonitor stop timed out."),
        }
    }
}
