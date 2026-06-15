use crate::ipc::StopResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateReport {
    current_version: String,
    latest_version: Option<String>,
    kind: UpdateReportKind,
    stop_result: StopResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateReportKind {
    UpToDate,
    DryRun,
    Updated,
}

impl UpdateReport {
    pub(in crate::update) fn up_to_date(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: None,
            kind: UpdateReportKind::UpToDate,
            stop_result: StopResult::NotRunning,
        }
    }

    pub(in crate::update) fn dry_run(
        current_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: current_version.into(),
            latest_version: Some(latest_version.into()),
            kind: UpdateReportKind::DryRun,
            stop_result: StopResult::NotRunning,
        }
    }

    pub(in crate::update) fn updated(
        previous_version: impl Into<String>,
        latest_version: impl Into<String>,
    ) -> Self {
        Self {
            current_version: previous_version.into(),
            latest_version: Some(latest_version.into()),
            kind: UpdateReportKind::Updated,
            stop_result: StopResult::NotRunning,
        }
    }

    pub(in crate::update) fn with_stop_result(mut self, result: StopResult) -> Self {
        self.stop_result = result;
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

        match self.stop_result {
            StopResult::NotRunning => update_line,
            StopResult::Stopped => format!("{update_line}\nRestarted xbattery monitor."),
            StopResult::TimedOut => format!("{update_line}\nMonitor stop timed out."),
        }
    }
}
