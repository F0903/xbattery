use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, single_instance::SingleInstance};

use super::{NamedEvent, StopResult};

const STOP_WAIT_SLICE: Duration = Duration::from_millis(100);

pub fn request_stop(
    mutex_name: &str,
    stop_event_name: &str,
    timeout: Duration,
) -> AppResult<StopResult> {
    if acquire_mutex(mutex_name)?.is_some() {
        return Ok(StopResult::NotRunning);
    }

    NamedEvent::open_or_create(stop_event_name)?.signal()?;
    wait_for_exit(mutex_name, timeout)
}

fn wait_for_exit(mutex_name: &str, timeout: Duration) -> AppResult<StopResult> {
    let deadline = Instant::now() + timeout;

    loop {
        if acquire_mutex(mutex_name)?.is_some() {
            return Ok(StopResult::Stopped);
        }

        if Instant::now() >= deadline {
            return Ok(StopResult::TimedOut);
        }

        thread::sleep(STOP_WAIT_SLICE);
    }
}

fn acquire_mutex(mutex_name: &str) -> AppResult<Option<SingleInstance>> {
    SingleInstance::acquire(mutex_name)
}
