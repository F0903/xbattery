use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
        System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject},
    },
    core::HSTRING,
};

use crate::AppResult;

use super::MONITOR_STOP_EVENT_NAME;

pub struct MonitorStopEvent {
    handle: HANDLE,
}

impl MonitorStopEvent {
    pub fn open_or_create() -> AppResult<Self> {
        let name = HSTRING::from(MONITOR_STOP_EVENT_NAME);
        let handle = unsafe { CreateEventW(None, true, false, &name)? };

        Ok(Self { handle })
    }

    pub fn reset(&self) -> AppResult<()> {
        unsafe {
            ResetEvent(self.handle)?;
        }

        Ok(())
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

impl Drop for MonitorStopEvent {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
