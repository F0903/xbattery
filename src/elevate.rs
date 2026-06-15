use std::env;

use windows::{
    Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    core::{HSTRING, PCWSTR},
};

use crate::AppResult;

pub fn relaunch_current_exe_as_admin(parameters: &str) -> AppResult<()> {
    let exe = env::current_exe()?;
    let exe = HSTRING::from(exe.as_os_str());
    let operation = HSTRING::from("runas");
    let parameters = HSTRING::from(parameters);

    let result = unsafe {
        ShellExecuteW(
            None,
            &operation,
            &exe,
            &parameters,
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 as isize <= 32 {
        return Err(format!(
            "failed to start elevated xbattery installer; ShellExecuteW returned {}",
            result.0 as isize
        )
        .into());
    }

    Ok(())
}
