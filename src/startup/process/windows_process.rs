use std::path::Path;

use crate::{AppResult, startup::paths::StartupPaths};

use super::{process_handle::ProcessHandle, snapshot_handle};

use windows::Win32::System::Threading::{
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

pub(super) fn terminate_processes_for_exe(exe_path: &Path) -> AppResult<u32> {
    let mut terminated_processes = 0;
    let current_process_id = unsafe { windows::Win32::System::Threading::GetCurrentProcessId() };

    for process_id in snapshot_handle::process_ids()? {
        if process_id == current_process_id {
            continue;
        }

        let Ok(query_process) = ProcessHandle::open(process_id, PROCESS_QUERY_LIMITED_INFORMATION)
        else {
            continue;
        };

        let Some(process_path) = query_process.image_path() else {
            continue;
        };

        if !StartupPaths::same_path(&process_path, exe_path) {
            continue;
        }

        let process = ProcessHandle::open(process_id, PROCESS_TERMINATE | PROCESS_SYNCHRONIZE)
            .map_err(|error| {
                format!("failed to open xbattery process {process_id} for termination: {error}")
            })?;
        process.terminate(process_id)?;
        terminated_processes += 1;
    }

    Ok(terminated_processes)
}
