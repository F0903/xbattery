#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use xbattery::AppResult;

fn main() -> AppResult<()> {
    init_com()?;
    cli::run(std::env::args().skip(1))
}

fn init_com() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}
