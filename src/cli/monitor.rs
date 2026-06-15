use xbattery::{
    AppResult,
    config::{self, AppConfig},
    controller::service::ControllerService,
    ipc::{BACKGROUND_INSTANCE_MUTEX_NAME, BACKGROUND_INSTANCE_STOP_EVENT_NAME, NamedEvent},
    notifier::ToastNotifier,
    single_instance::SingleInstance,
    update,
};

pub(super) fn run() -> AppResult<()> {
    let Some(_guard) = SingleInstance::acquire(BACKGROUND_INSTANCE_MUTEX_NAME)? else {
        println!("xbattery monitor is already running.");
        return Ok(());
    };

    let stop_event = NamedEvent::open_or_create(BACKGROUND_INSTANCE_STOP_EVENT_NAME)?;
    stop_event.reset()?;

    let loaded_config = AppConfig::load_with_source()?;
    let config_watcher = loaded_config.path.map(config::watch_config);
    let config = loaded_config.config;
    let notifier = ToastNotifier::new(config.toast_config());
    let reload_notifier = notifier.clone();
    let update_handle = update::start_background_checks(config.updates.clone(), notifier.clone())?;

    let mut service = ControllerService::new(notifier, config.controller_service_config()?);
    service.run_until_ctrl_c_or_reconfigure(|| stop_event.is_signaled(), {
        let notifier = reload_notifier;
        move || {
            let mut latest = None;

            if let Some(config_watcher) = &config_watcher {
                while let Ok(config) = config_watcher.try_recv() {
                    notifier.set_config(config.toast_config())?;
                    update_handle.update_config(config.updates.clone())?;
                    latest = Some(config.controller_service_config()?);
                }
            }

            Ok(latest)
        }
    })
}
