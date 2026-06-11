#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    GameInput,
    XInput,
    WinRT,
}

impl BackendKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::GameInput => "GameInput",
            Self::XInput => "XInput",
            Self::WinRT => "WinRT Gamepad",
        }
    }
}
