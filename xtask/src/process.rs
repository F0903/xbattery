use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::Result;

pub fn repo_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest directory has no parent")?
        .to_path_buf())
}

pub fn ensure_exists(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(format!("Expected {label} was not found: {}", path.display()).into())
    }
}

pub fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed with status {status:?}").into())
    }
}
