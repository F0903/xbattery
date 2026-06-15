mod automatic_update_handle;
mod automatic_update_loop;
mod notify;

use crate::{AppResult, config::UpdatesConfig, notifier::ToastNotifier};

pub use automatic_update_handle::AutomaticUpdateHandle;

pub fn start_background_checks(
    config: UpdatesConfig,
    notifier: ToastNotifier,
) -> AppResult<AutomaticUpdateHandle> {
    automatic_update_loop::AutomaticUpdateLoop::start(config, notifier)
}
