use xbattery::{
    AppResult, config::AppConfig, controller::service::ControllerService, notifier::ToastNotifier,
    single_instance::SingleInstance,
};

pub(super) fn run() -> AppResult<()> {
    let Some(_guard) = SingleInstance::acquire("Local\\xbattery-monitor")? else {
        println!("xbattery monitor is already running.");
        return Ok(());
    };

    let config = AppConfig::load()?;
    let mut service = ControllerService::new(
        ToastNotifier::new(config.toast_config()),
        config.controller_service_config()?,
    );
    service.run_until_ctrl_c()
}
