use std::{path::Path, process::Command};

use crate::AppResult;

pub(super) fn spawn_update_process(installed_exe: &Path) -> AppResult<()> {
    let mut command = Command::new(installed_exe);
    command.arg("update");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.spawn()?;
    Ok(())
}
