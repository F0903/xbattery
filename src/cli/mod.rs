mod args;
mod command;
#[cfg(debug_assertions)]
mod diagnostics;
mod help;
mod install;
mod monitor;
#[cfg(debug_assertions)]
mod rumble;
#[cfg(debug_assertions)]
mod toast;
mod update;

use clap::Parser;
use xbattery::{AppResult, launch_context::LaunchContext};

use self::{
    args::argv,
    command::{Cli, Command},
    help::print_help,
};

pub fn run(args: impl IntoIterator<Item = String>) -> AppResult<()> {
    let cli = Cli::parse_from(argv(args));

    match cli.command {
        None if LaunchContext::current().is_likely_explorer_launch() => {
            install::install_interactive()
        }
        None => print_help(None),
        Some(Command::Help { command }) => print_help(command.as_deref()),
        Some(Command::Install { force }) => install::install(false, force),
        Some(Command::InstallElevated) => install::install_elevated_retry(),
        Some(Command::Uninstall) => install::uninstall(false),
        Some(Command::Status) => install::status(),
        Some(Command::CheckUpdate) => update::check(),
        Some(Command::Update { dry_run }) => update::run(dry_run),
        Some(Command::Monitor) => monitor::run(),
        #[cfg(debug_assertions)]
        Some(Command::Probe) => diagnostics::probe(),
        #[cfg(debug_assertions)]
        Some(Command::GameinputProbe) => diagnostics::gameinput_probe(),
        #[cfg(debug_assertions)]
        Some(Command::GameinputWatch) => diagnostics::gameinput_watch(),
        #[cfg(debug_assertions)]
        Some(Command::RumbleTest) => rumble::test(),
        #[cfg(debug_assertions)]
        Some(Command::RumbleTestThresholds) => rumble::test_thresholds(),
        #[cfg(debug_assertions)]
        Some(Command::ToastTest) => toast::test(),
        #[cfg(debug_assertions)]
        Some(Command::ToastTestHigh) => toast::test_high(),
        #[cfg(debug_assertions)]
        Some(Command::ToastTestUrgent) => toast::test_urgent(),
        #[cfg(debug_assertions)]
        Some(Command::NotificationPreview) => toast::preview(),
    }
}
