use std::path::Path;

use crate::AppResult;

#[cfg(windows)]
mod process_handle;
#[cfg(windows)]
mod snapshot_handle;
#[cfg(windows)]
mod windows_process;

#[cfg(windows)]
pub(super) fn terminate_processes_for_exe(exe_path: &Path) -> AppResult<u32> {
    windows_process::terminate_processes_for_exe(exe_path)
}

#[cfg(not(windows))]
pub(super) fn terminate_processes_for_exe(_exe_path: &Path) -> AppResult<u32> {
    Ok(0)
}
