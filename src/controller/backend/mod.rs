mod gameinput;
mod win_rt;
mod xinput;

pub use gameinput::GameInputBackend;
pub(crate) use gameinput::{GameInputEvent, GameInputEventStream};
pub use win_rt::WinRTBackend;
pub use xinput::XInputBackend;
