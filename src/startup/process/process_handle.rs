use std::path::{Path, PathBuf};

use crate::{AppResult, startup::paths::StartupPaths};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
            QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        },
    },
    core::PWSTR,
};

use super::snapshot_handle;

const PROCESS_TERMINATION_EXIT_CODE: u32 = 1;
const PROCESS_TERMINATION_WAIT_MILLIS: u32 = 5_000;

pub(in crate::startup) struct ProcessHandle {
    process_id: u32,
    handle: HANDLE,
}

impl ProcessHandle {
    pub(in crate::startup) fn for_exe_path(exe_path: &Path) -> AppResult<Vec<Self>> {
        let mut processes = Vec::new();
        let current_process_id = unsafe { GetCurrentProcessId() };

        for process_id in snapshot_handle::process_ids()? {
            if process_id == current_process_id || !process_matches_exe_path(process_id, exe_path) {
                continue;
            }

            processes.push(
                Self::open(process_id, PROCESS_TERMINATE | PROCESS_SYNCHRONIZE).map_err(
                    |error| {
                        format!(
                            "failed to open xbattery process {process_id} for termination: {error}"
                        )
                    },
                )?,
            );
        }

        Ok(processes)
    }

    fn open(process_id: u32, access: PROCESS_ACCESS_RIGHTS) -> windows::core::Result<Self> {
        let handle = unsafe { OpenProcess(access, false, process_id)? };

        Ok(Self { process_id, handle })
    }

    fn image_path(&self) -> Option<PathBuf> {
        let mut buffer = vec![0; 32_768];
        let mut buffer_len = buffer.len() as u32;

        unsafe {
            QueryFullProcessImageNameW(
                self.handle,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut buffer_len,
            )
            .ok()?;
        }

        Some(PathBuf::from(String::from_utf16_lossy(
            &buffer[..buffer_len as usize],
        )))
    }

    pub(in crate::startup) fn terminate(&self) -> AppResult<()> {
        unsafe {
            TerminateProcess(self.handle, PROCESS_TERMINATION_EXIT_CODE)?;
        }

        match unsafe { WaitForSingleObject(self.handle, PROCESS_TERMINATION_WAIT_MILLIS) } {
            WAIT_OBJECT_0 => Ok(()),
            WAIT_TIMEOUT => Err(format!(
                "timed out waiting for xbattery process {} to exit",
                self.process_id
            )
            .into()),
            wait_result => Err(format!(
                "failed waiting for xbattery process {}: {wait_result:?}",
                self.process_id
            )
            .into()),
        }
    }
}

fn process_matches_exe_path(process_id: u32, exe_path: &Path) -> bool {
    let Ok(process) = ProcessHandle::open(process_id, PROCESS_QUERY_LIMITED_INFORMATION) else {
        return false;
    };

    let Some(process_path) = process.image_path() else {
        return false;
    };

    StartupPaths::same_path(&process_path, exe_path)
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
