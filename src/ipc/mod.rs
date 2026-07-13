mod named_event;
mod stop_signal;

pub use named_event::NamedEvent;
pub use stop_signal::{StopResult, request_stop};

pub const BACKGROUND_INSTANCE_MUTEX_NAME: &str = "Local\\xbattery-monitor";
pub const BACKGROUND_INSTANCE_STOP_EVENT_NAME: &str = "Local\\xbattery-monitor-stop";
