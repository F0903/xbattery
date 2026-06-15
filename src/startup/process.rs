use std::path::{Path, PathBuf};

use crate::AppResult;

use super::paths::StartupPaths;

#[cfg(windows)]
pub(super) fn terminate_processes_for_exe(exe_path: &Path) -> AppResult<u32> {
    windows_process::terminate_processes_for_exe(exe_path)
}

#[cfg(not(windows))]
pub(super) fn terminate_processes_for_exe(_exe_path: &Path) -> AppResult<u32> {
    Ok(0)
}

#[cfg(windows)]
mod windows_process {
    use super::*;
    use windows::{
        Win32::{
            Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
            System::{
                Diagnostics::ToolHelp::{
                    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                    TH32CS_SNAPPROCESS,
                },
                Threading::{
                    GetCurrentProcessId, OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
                    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
                    QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
                },
            },
        },
        core::PWSTR,
    };

    const PROCESS_TERMINATION_EXIT_CODE: u32 = 1;
    const PROCESS_TERMINATION_WAIT_MILLIS: u32 = 5_000;

    pub(super) fn terminate_processes_for_exe(exe_path: &Path) -> AppResult<u32> {
        let mut terminated_processes = 0;
        let current_process_id = unsafe { GetCurrentProcessId() };

        for process_id in process_ids()? {
            if process_id == current_process_id {
                continue;
            }

            let Ok(query_process) =
                ProcessHandle::open(process_id, PROCESS_QUERY_LIMITED_INFORMATION)
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

    fn process_ids() -> AppResult<Vec<u32>> {
        let snapshot = SnapshotHandle::create()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut process_ids = Vec::new();

        unsafe {
            Process32FirstW(snapshot.raw(), &mut entry)?;
        }

        loop {
            process_ids.push(entry.th32ProcessID);

            if unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_err() {
                break;
            }
        }

        Ok(process_ids)
    }

    struct SnapshotHandle {
        handle: HANDLE,
    }

    impl SnapshotHandle {
        fn create() -> AppResult<Self> {
            Ok(Self {
                handle: unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? },
            })
        }

        fn raw(&self) -> HANDLE {
            self.handle
        }
    }

    impl Drop for SnapshotHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }

    struct ProcessHandle {
        handle: HANDLE,
    }

    impl ProcessHandle {
        fn open(process_id: u32, access: PROCESS_ACCESS_RIGHTS) -> windows::core::Result<Self> {
            let handle = unsafe { OpenProcess(access, false, process_id)? };

            Ok(Self { handle })
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

        fn terminate(&self, process_id: u32) -> AppResult<()> {
            unsafe {
                TerminateProcess(self.handle, PROCESS_TERMINATION_EXIT_CODE)?;
            }

            match unsafe { WaitForSingleObject(self.handle, PROCESS_TERMINATION_WAIT_MILLIS) } {
                WAIT_OBJECT_0 => Ok(()),
                WAIT_TIMEOUT => Err(format!(
                    "timed out waiting for xbattery process {process_id} to exit"
                )
                .into()),
                wait_result => Err(format!(
                    "failed waiting for xbattery process {process_id}: {wait_result:?}"
                )
                .into()),
            }
        }
    }

    impl Drop for ProcessHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
