use std::{
    os::windows::ffi::OsStrExt,
    sync::{Mutex, PoisonError},
};

use windows::{
    Win32::Media::Audio::{
        PlaySoundW, SND_ASYNC, SND_FILENAME, SND_MEMORY, SND_NODEFAULT, SND_SYNC, SND_SYSTEM,
    },
    core::PCWSTR,
};

use crate::{AppResult, audio::AudioClip};

static ACTIVE_ASYNC_CLIP: Mutex<Option<AudioClip>> = Mutex::new(None);

#[derive(Clone, Copy)]
enum PlaybackMode {
    Async,
    Blocking,
}

pub fn play(clip: &AudioClip) -> AppResult<()> {
    play_with_mode(clip, PlaybackMode::Async)
}

pub fn play_blocking(clip: &AudioClip) -> AppResult<()> {
    play_with_mode(clip, PlaybackMode::Blocking)
}

fn play_with_mode(clip: &AudioClip, mode: PlaybackMode) -> AppResult<()> {
    // PlaySound is process-global. A shared lock prevents this module's calls from racing, and
    // retaining the active clip keeps SND_MEMORY data alive for asynchronous playback.
    let mut active_clip = ACTIVE_ASYNC_CLIP
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let mode_flag = match mode {
        PlaybackMode::Async => SND_ASYNC,
        PlaybackMode::Blocking => SND_SYNC,
    };

    let played = match clip {
        AudioClip::File(path) => {
            let path_wide = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            let flags = SND_FILENAME | SND_NODEFAULT | SND_SYSTEM | mode_flag;

            unsafe { PlaySoundW(PCWSTR(path_wide.as_ptr()), None, flags) }.as_bool()
        }
        AudioClip::WavBytes(bytes) => {
            let flags = SND_MEMORY | SND_NODEFAULT | SND_SYSTEM | mode_flag;

            unsafe { PlaySoundW(PCWSTR(bytes.as_ptr().cast()), None, flags) }.as_bool()
        }
    };

    if !played {
        return Err(format!("failed to play audio clip {clip}").into());
    }

    match mode {
        PlaybackMode::Async => *active_clip = Some(clip.clone()),
        PlaybackMode::Blocking => *active_clip = None,
    }

    Ok(())
}
