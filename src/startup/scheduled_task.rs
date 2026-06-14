use std::{
    path::Path,
    process::{Command, Stdio},
};

use super::{StartupAccessDenied, TASK_NAME};
use crate::AppResult;

#[derive(Clone, Debug)]
pub(super) struct ScheduledTask {
    name: &'static str,
}

impl ScheduledTask {
    pub(super) fn xbattery() -> Self {
        Self { name: TASK_NAME }
    }

    pub(super) fn create(&self, executable: &Path) -> AppResult<()> {
        let task_command = format!("\"{}\" monitor", executable.display());
        run_schtasks(
            "create the xbattery startup task",
            [
                "/Create",
                "/TN",
                self.name,
                "/SC",
                "ONLOGON",
                "/TR",
                &task_command,
                "/F",
            ],
        )
    }

    pub(super) fn delete(&self) -> AppResult<bool> {
        if !self.exists() {
            return Ok(false);
        }

        run_schtasks(
            "remove the xbattery startup task",
            ["/Delete", "/TN", self.name, "/F"],
        )?;
        Ok(true)
    }

    pub(super) fn exists(&self) -> bool {
        Command::new("schtasks.exe")
            .args(["/Query", "/TN", self.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

fn run_schtasks<'a>(
    operation: &'static str,
    args: impl IntoIterator<Item = &'a str>,
) -> AppResult<()> {
    let output = Command::new("schtasks.exe").args(args).output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = format!(
        "schtasks.exe failed with status {}. stdout: {} stderr: {}",
        output.status, stdout, stderr
    );

    if is_access_denied(&stdout) || is_access_denied(&stderr) {
        return Err(StartupAccessDenied::new(operation, details).into());
    }

    Err(details.into())
}

fn is_access_denied(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("access is denied")
        || text.contains("access denied")
        || text.contains("0x80070005")
}
