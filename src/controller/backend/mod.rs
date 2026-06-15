mod backend_event;
mod backend_kind;
mod gameinput;
mod traits;
mod win_rt;
mod xinput;

pub use backend_event::{BackendEvent, BackendEventStream};
pub use backend_kind::BackendKind;
pub use gameinput::GameInputBackend;
pub use traits::{BatteryBackend, EventBackend, InputBackend, RumbleBackend};
pub use win_rt::WinRTBackend;
pub use xinput::XInputBackend;
