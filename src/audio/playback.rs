use std::{os::windows::ffi::OsStrExt, path::Path};

use windows::{
    Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT, SND_SYSTEM},
    core::PCWSTR,
};

use crate::AppResult;

pub fn play_file(path: &Path) -> AppResult<()> {
    play_file_with_mode(path, true)
}

pub fn play_file_blocking(path: &Path) -> AppResult<()> {
    play_file_with_mode(path, false)
}

fn play_file_with_mode(path: &Path, asynchronous: bool) -> AppResult<()> {
    let path_wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let mut flags = SND_FILENAME | SND_NODEFAULT | SND_SYSTEM;
    if asynchronous {
        flags |= SND_ASYNC;
    }

    let played = unsafe { PlaySoundW(PCWSTR(path_wide.as_ptr()), None, flags) }.as_bool();

    if played {
        Ok(())
    } else {
        Err(format!("failed to play audio file {}", path.display()).into())
    }
}
