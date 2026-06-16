use std::{fs::OpenOptions, os::windows::io::IntoRawHandle};

use windows::Win32::{
    Foundation::HANDLE,
    System::Console::{
        ATTACH_PARENT_PROCESS, AttachConsole, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
        STD_OUTPUT_HANDLE, SetStdHandle,
    },
};

use crate::AppResult;

#[derive(Clone, Copy, Debug)]
pub struct Console;

impl Console {
    pub fn attach_to_parent() -> AppResult<()> {
        if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_err() {
            return Ok(());
        }

        set_standard_handle(STD_INPUT_HANDLE, "CONIN$", true, false)?;
        set_standard_handle(STD_OUTPUT_HANDLE, "CONOUT$", false, true)?;
        set_standard_handle(STD_ERROR_HANDLE, "CONOUT$", false, true)?;
        Ok(())
    }
}

fn set_standard_handle(
    standard_handle: windows::Win32::System::Console::STD_HANDLE,
    path: &str,
    read: bool,
    write: bool,
) -> AppResult<()> {
    let file = OpenOptions::new().read(read).write(write).open(path)?;
    let handle = HANDLE(file.into_raw_handle());

    unsafe {
        SetStdHandle(standard_handle, handle)?;
    }

    Ok(())
}
