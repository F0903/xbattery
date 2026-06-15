use windows::Win32::System::Console::GetConsoleProcessList;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchContext {
    console_process_count: Option<u32>,
}

impl LaunchContext {
    pub fn current() -> Self {
        Self {
            console_process_count: current_console_process_count(),
        }
    }

    // Checks if we were likely launched from the File Explorer or Desktop.
    pub fn is_likely_explorer_launch(self) -> bool {
        matches!(self.console_process_count, Some(1))
    }
}

fn current_console_process_count() -> Option<u32> {
    let mut process_ids = [0; 8];
    let count = unsafe { GetConsoleProcessList(&mut process_ids) };

    if count == 0 { None } else { Some(count) }
}
