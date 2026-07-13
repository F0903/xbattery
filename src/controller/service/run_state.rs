use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::AppResult;

pub(super) struct RunState {
    running: Arc<AtomicBool>,
}

impl RunState {
    pub(super) fn new() -> AppResult<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_signal = Arc::clone(&running);

        ctrlc::set_handler(move || {
            running_signal.store(false, Ordering::SeqCst);
        })?;

        Ok(Self { running })
    }

    pub(super) fn active(&self, should_stop: &impl Fn() -> bool) -> bool {
        self.running.load(Ordering::SeqCst) && !should_stop()
    }
}
