use super::super::constants::{
    GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US, GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE,
};
use super::abi::IGameInput;

/// Unregisters a callback and waits for any in-flight invocation to finish.
///
/// # Safety
///
/// `game_input` must be a valid, non-null `IGameInput` pointer, and `token` must either be the
/// invalid sentinel or a callback token registered on that interface.
pub(super) unsafe fn unregister_callback(game_input: *mut IGameInput, token: u64) -> bool {
    if token == GAMEINPUT_INVALID_CALLBACK_TOKEN_VALUE {
        return true;
    }

    unsafe {
        ((*(*game_input).vtbl).UnregisterCallback)(
            game_input,
            token,
            GAMEINPUT_CALLBACK_UNREGISTER_TIMEOUT_US,
        )
    }
}
