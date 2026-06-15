mod stop_event;
mod stop_monitor;
mod stop_result;

pub use stop_event::MonitorStopEvent;
pub use stop_monitor::stop_monitor;
pub use stop_result::MonitorStopResult;

pub const MONITOR_MUTEX_NAME: &str = "Local\\xbattery-monitor";

const MONITOR_STOP_EVENT_NAME: &str = "Local\\xbattery-monitor-stop";
