use xbattery::{
    AppResult,
    config::AppConfig,
    controller::service::ControllerService,
    monitor_control::{MONITOR_MUTEX_NAME, MonitorStopEvent},
    notifier::ToastNotifier,
    single_instance::SingleInstance,
};

pub(super) fn run() -> AppResult<()> {
    let Some(_guard) = SingleInstance::acquire(MONITOR_MUTEX_NAME)? else {
        println!("xbattery monitor is already running.");
        return Ok(());
    };

    let stop_event = MonitorStopEvent::open_or_create()?;
    stop_event.reset()?;

    let config = AppConfig::load()?;
    let mut service = ControllerService::new(
        ToastNotifier::new(config.toast_config()),
        config.controller_service_config()?,
    );
    service.run_until_ctrl_c_or(|| stop_event.is_signaled())
}
