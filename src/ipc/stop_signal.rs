use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{AppResult, single_instance::SingleInstance};

use super::NamedEvent;

const STOP_WAIT_SLICE: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopResult {
    NotRunning,
    Stopped,
    TimedOut,
}

pub fn request_stop(
    mutex_name: &str,
    stop_event_name: &str,
    timeout: Duration,
) -> AppResult<StopResult> {
    request_stop_inner(mutex_name, stop_event_name, timeout, || {})
}

fn request_stop_inner(
    mutex_name: &str,
    stop_event_name: &str,
    timeout: Duration,
    on_signaled: impl FnOnce(),
) -> AppResult<StopResult> {
    if SingleInstance::acquire(mutex_name)?.is_some() {
        return Ok(StopResult::NotRunning);
    }

    let stop_event = NamedEvent::open_or_create(stop_event_name)?;
    stop_event.signal()?;
    on_signaled();
    let result = wait_for_exit(mutex_name, timeout);
    drop(stop_event);
    result
}

fn wait_for_exit(mutex_name: &str, timeout: Duration) -> AppResult<StopResult> {
    let deadline = Instant::now() + timeout;

    loop {
        if SingleInstance::acquire(mutex_name)?.is_some() {
            return Ok(StopResult::Stopped);
        }

        if Instant::now() >= deadline {
            return Ok(StopResult::TimedOut);
        }

        thread::sleep(STOP_WAIT_SLICE);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
        thread,
        time::Duration,
    };

    use crate::{ipc::NamedEvent, single_instance::SingleInstance};

    use super::{StopResult, request_stop_inner};

    static NEXT_NAME: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn keeps_the_stop_event_alive_until_a_starting_monitor_opens_it() {
        let id = NEXT_NAME.fetch_add(1, Ordering::Relaxed);
        let name = format!("{}-{id}", std::process::id());
        let mutex_name = format!("Local\\xbattery-test-monitor-{name}");
        let event_name = format!("Local\\xbattery-test-stop-{name}");
        let monitor = SingleInstance::acquire(&mutex_name)
            .unwrap()
            .expect("test mutex must be unique");
        let request_mutex = mutex_name.clone();
        let request_event = event_name.clone();
        let (signaled_tx, signaled_rx) = mpsc::channel();

        let request = thread::spawn(move || {
            request_stop_inner(
                &request_mutex,
                &request_event,
                Duration::from_secs(2),
                || signaled_tx.send(()).unwrap(),
            )
        });

        signaled_rx.recv_timeout(Duration::from_secs(1)).unwrap();

        let stop_event = NamedEvent::open_or_create(&event_name).unwrap();
        let signal_survived = stop_event.is_signaled();
        drop(monitor);
        let result = request.join().unwrap().unwrap();

        assert!(signal_survived);
        assert_eq!(result, StopResult::Stopped);
    }
}
