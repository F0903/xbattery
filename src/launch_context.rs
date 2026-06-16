use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Console::GetConsoleProcessList,
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::GetCurrentProcessId,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchContext {
    console_process_count: Option<u32>,
    parent_process_name: Option<String>,
}

impl LaunchContext {
    pub fn current() -> Self {
        Self {
            console_process_count: current_console_process_count(),
            parent_process_name: current_parent_process_name(),
        }
    }

    pub fn has_console(&self) -> bool {
        self.console_process_count.is_some()
    }

    // Checks if we were likely launched from the File Explorer or Desktop.
    pub fn is_likely_explorer_launch(&self) -> bool {
        matches!(self.console_process_count, Some(1))
            || self
                .parent_process_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case("explorer.exe"))
    }
}

fn current_console_process_count() -> Option<u32> {
    let mut process_ids = [0; 8];
    let count = unsafe { GetConsoleProcessList(&mut process_ids) };

    if count == 0 { None } else { Some(count) }
}

fn current_parent_process_name() -> Option<String> {
    let current_process_id = unsafe { GetCurrentProcessId() };
    let processes = process_entries()?;
    let parent_process_id = processes
        .iter()
        .find(|process| process.id == current_process_id)?
        .parent_id;

    processes
        .iter()
        .find(|process| process.id == parent_process_id)
        .map(|process| process.name.clone())
}

fn process_entries() -> Option<Vec<ProcessEntry>> {
    let snapshot = SnapshotHandle::create().ok()?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut processes = Vec::new();

    unsafe {
        Process32FirstW(snapshot.raw(), &mut entry).ok()?;
    }

    loop {
        processes.push(ProcessEntry::from_entry(&entry));

        if unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_err() {
            break;
        }
    }

    Some(processes)
}

#[derive(Clone, Debug)]
struct ProcessEntry {
    id: u32,
    parent_id: u32,
    name: String,
}

impl ProcessEntry {
    fn from_entry(entry: &PROCESSENTRY32W) -> Self {
        Self {
            id: entry.th32ProcessID,
            parent_id: entry.th32ParentProcessID,
            name: process_name(&entry.szExeFile),
        }
    }
}

fn process_name(raw_name: &[u16]) -> String {
    let end = raw_name
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(raw_name.len());

    String::from_utf16_lossy(&raw_name[..end])
}

struct SnapshotHandle {
    handle: HANDLE,
}

impl SnapshotHandle {
    fn create() -> windows::core::Result<Self> {
        Ok(Self {
            handle: unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? },
        })
    }

    fn raw(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
