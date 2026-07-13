mod gameinput;
#[cfg(debug_assertions)]
mod win_rt;
mod xinput;

pub use gameinput::GameInputBackend;
pub(crate) use gameinput::{GameInputEvent, GameInputEventStream};
#[cfg(debug_assertions)]
pub use win_rt::WinRTBackend;
pub use xinput::XInputBackend;
