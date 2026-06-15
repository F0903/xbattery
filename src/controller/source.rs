use super::backend::BackendKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControllerSource {
    GameInput,
    XInput,
    WinRT,
}

impl ControllerSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::GameInput => "GameInput",
            Self::XInput => "XInput",
            Self::WinRT => "Windows.Gaming.Input",
        }
    }

    pub(super) fn backend_kind(self) -> BackendKind {
        match self {
            Self::GameInput => BackendKind::GameInput,
            Self::XInput => BackendKind::XInput,
            Self::WinRT => BackendKind::WinRT,
        }
    }
}
