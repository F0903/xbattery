use super::raw;

#[derive(Clone, Debug)]
pub struct GameInputDiagnosticSnapshot {
    pub timestamp: u64,
    pub current_status: String,
    pub previous_status: String,
}

impl GameInputDiagnosticSnapshot {
    pub(super) fn from_snapshot(snapshot: raw::GameInputDeviceSnapshot) -> Self {
        Self {
            timestamp: snapshot.timestamp,
            current_status: snapshot.current_status_description(),
            previous_status: snapshot.previous_status_description(),
        }
    }
}
