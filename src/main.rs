#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use xbattery::{AppResult, console::Console, launch_context::LaunchContext};

fn main() -> AppResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let launch_context = LaunchContext::current();

    if should_attach_console(&args, &launch_context) {
        Console::attach_to_parent()?;
    }

    init_com()?;
    cli::run(args)
}

fn should_attach_console(args: &[String], launch_context: &LaunchContext) -> bool {
    if launch_context.has_console() {
        return false;
    }

    if args.first().is_some_and(|arg| runs_without_console(arg)) {
        return false;
    }

    !args.is_empty() || !launch_context.is_likely_explorer_launch()
}

fn runs_without_console(command: &str) -> bool {
    command.eq_ignore_ascii_case("monitor") || command.eq_ignore_ascii_case("install-elevated")
}

fn init_com() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}
