use std::{
    thread,
    time::{Duration, Instant},
};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
        System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject},
    },
    core::HSTRING,
};

use crate::{AppResult, single_instance::SingleInstance};

pub const MONITOR_MUTEX_NAME: &str = "Local\\xbattery-monitor";

const MONITOR_STOP_EVENT_NAME: &str = "Local\\xbattery-monitor-stop";
const STOP_WAIT_SLICE: Duration = Duration::from_millis(100);

pub struct MonitorStopEvent {
    handle: HANDLE,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorStopResult {
    NotRunning,
    Stopped,
    TimedOut,
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

pub fn stop_monitor(timeout: Duration) -> AppResult<MonitorStopResult> {
    if acquire_monitor_mutex()?.is_some() {
        return Ok(MonitorStopResult::NotRunning);
    }

    MonitorStopEvent::open_or_create()?.signal()?;
    wait_for_monitor_exit(timeout)
}

fn wait_for_monitor_exit(timeout: Duration) -> AppResult<MonitorStopResult> {
    let deadline = Instant::now() + timeout;

    loop {
        if acquire_monitor_mutex()?.is_some() {
            return Ok(MonitorStopResult::Stopped);
        }

        if Instant::now() >= deadline {
            return Ok(MonitorStopResult::TimedOut);
        }

        thread::sleep(STOP_WAIT_SLICE);
    }
}

fn acquire_monitor_mutex() -> AppResult<Option<SingleInstance>> {
    SingleInstance::acquire(MONITOR_MUTEX_NAME)
}
