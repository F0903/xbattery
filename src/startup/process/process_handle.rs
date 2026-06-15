use std::path::PathBuf;

use crate::AppResult;

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{
            OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32, QueryFullProcessImageNameW,
            TerminateProcess, WaitForSingleObject,
        },
    },
    core::PWSTR,
};

const PROCESS_TERMINATION_EXIT_CODE: u32 = 1;
const PROCESS_TERMINATION_WAIT_MILLIS: u32 = 5_000;

pub(super) struct ProcessHandle {
    handle: HANDLE,
}

impl ProcessHandle {
    pub(super) fn open(
        process_id: u32,
        access: PROCESS_ACCESS_RIGHTS,
    ) -> windows::core::Result<Self> {
        let handle = unsafe { OpenProcess(access, false, process_id)? };

        Ok(Self { handle })
    }

    pub(super) fn image_path(&self) -> Option<PathBuf> {
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

    pub(super) fn terminate(&self, process_id: u32) -> AppResult<()> {
        unsafe {
            TerminateProcess(self.handle, PROCESS_TERMINATION_EXIT_CODE)?;
        }

        match unsafe { WaitForSingleObject(self.handle, PROCESS_TERMINATION_WAIT_MILLIS) } {
            WAIT_OBJECT_0 => Ok(()),
            WAIT_TIMEOUT => {
                Err(format!("timed out waiting for xbattery process {process_id} to exit").into())
            }
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
