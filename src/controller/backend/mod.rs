mod controller_backend;
pub(crate) mod gameinput;
#[cfg(debug_assertions)]
pub(crate) mod win_rt;
mod windows;
pub(crate) mod xinput;

pub(crate) use controller_backend::{
    ControllerBackend, ControllerEventStream, ControllerStreamStatus,
};
pub(crate) use windows::WindowsControllerBackend;
