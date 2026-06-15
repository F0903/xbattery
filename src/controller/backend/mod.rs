mod backend_event;
mod backend_kind;
mod capability;
mod gameinput;
mod win_rt;
mod xinput;

pub use backend_event::{BackendEvent, BackendEventStream};
pub use backend_kind::BackendKind;
pub use capability::{ControllerBattery, ControllerEventInput, ControllerInput, ControllerRumbler};
pub use gameinput::GameInputBackend;
pub use win_rt::WinRTBackend;
pub use xinput::XInputBackend;
