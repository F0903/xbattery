use crate::AppResult;

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    },
};

pub(super) fn process_ids() -> AppResult<Vec<u32>> {
    let snapshot = SnapshotHandle::create()?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut process_ids = Vec::new();

    unsafe {
        Process32FirstW(snapshot.raw(), &mut entry)?;
    }

    loop {
        process_ids.push(entry.th32ProcessID);

        if unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_err() {
            break;
        }
    }

    Ok(process_ids)
}

struct SnapshotHandle {
    handle: HANDLE,
}

impl SnapshotHandle {
    fn create() -> AppResult<Self> {
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
