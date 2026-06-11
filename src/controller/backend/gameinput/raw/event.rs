use super::GameInputDeviceSnapshot;

#[derive(Clone, Debug)]
pub enum GameInputEvent {
    Device(GameInputDeviceSnapshot),
    Reading(GameInputDeviceSnapshot),
}

impl GameInputEvent {
    pub(super) fn device(snapshot: GameInputDeviceSnapshot) -> Self {
        Self::Device(snapshot)
    }

    pub(super) fn reading(snapshot: GameInputDeviceSnapshot) -> Self {
        Self::Reading(snapshot)
    }

    pub fn snapshot(&self) -> &GameInputDeviceSnapshot {
        match self {
            Self::Device(snapshot) | Self::Reading(snapshot) => snapshot,
        }
    }

    pub fn into_snapshot(self) -> GameInputDeviceSnapshot {
        match self {
            Self::Device(snapshot) | Self::Reading(snapshot) => snapshot,
        }
    }

    pub fn source_label(&self) -> &'static str {
        match self {
            Self::Device(_) => "device",
            Self::Reading(_) => "reading",
        }
    }
}
