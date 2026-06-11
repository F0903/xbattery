use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE},
        System::Threading::{CreateMutexW, ReleaseMutex},
    },
    core::HSTRING,
};

use crate::AppResult;

pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    pub fn acquire(name: &str) -> AppResult<Option<Self>> {
        let name = HSTRING::from(name);
        let handle = unsafe { CreateMutexW(None, true, &name)? };
        let already_running = unsafe { GetLastError() == ERROR_ALREADY_EXISTS };

        if already_running {
            unsafe {
                CloseHandle(handle)?;
            }
            Ok(None)
        } else {
            Ok(Some(Self { handle }))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}
