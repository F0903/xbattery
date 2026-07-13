use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
        System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
    },
    core::HSTRING,
};

use crate::AppResult;

pub struct NamedEvent {
    handle: HANDLE,
}

impl NamedEvent {
    pub fn open_or_create(name: &str) -> AppResult<Self> {
        let name = HSTRING::from(name);
        let handle = unsafe { CreateEventW(None, true, false, &name)? };

        Ok(Self { handle })
    }

    pub fn signal(&self) -> AppResult<()> {
        unsafe {
            SetEvent(self.handle)?;
        }

        Ok(())
    }

    pub fn is_signaled(&self) -> bool {
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_OBJECT_0 }
    }
}

impl Drop for NamedEvent {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
