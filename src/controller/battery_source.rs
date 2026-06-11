use super::{Controller, backend::ControllerBattery};

pub fn attach_battery_readings<B>(controllers: Vec<Controller>, battery: &B) -> Vec<Controller>
where
    B: ControllerBattery,
{
    let Ok(readings) = battery.battery_readings() else {
        return controllers;
    };

    if readings.len() != controllers.len() {
        return controllers;
    }

    let source = battery.backend_kind();
    controllers
        .into_iter()
        .zip(readings)
        .map(|(controller, reading)| controller.with_battery(source, reading))
        .collect()
}

pub fn attach_single_battery_reading<B>(controller: Controller, battery: &B) -> Controller
where
    B: ControllerBattery,
{
    let Ok(readings) = battery.battery_readings() else {
        return controller;
    };

    match readings.as_slice() {
        [reading] => controller.with_battery(battery.backend_kind(), *reading),
        _ => controller,
    }
}
