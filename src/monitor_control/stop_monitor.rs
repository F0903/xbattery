use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, single_instance::SingleInstance};

use super::{MONITOR_MUTEX_NAME, MonitorStopEvent, MonitorStopResult};

const STOP_WAIT_SLICE: Duration = Duration::from_millis(100);

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
