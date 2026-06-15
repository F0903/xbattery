#[cfg(debug_assertions)]
mod diagnostics;
mod install;
mod monitor;
#[cfg(debug_assertions)]
mod rumble;
#[cfg(debug_assertions)]
mod toast;
mod update;

use clap::{CommandFactory, Parser, Subcommand};
use xbattery::{AppResult, launch_context::LaunchContext};

#[derive(Parser)]
#[command(
    name = "xbattery",
    about = "Xbox controller battery notifications",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print help or help for a command.
    #[command(disable_help_flag = true)]
    Help { command: Option<String> },
    /// Install per-user startup task and start background monitor.
    #[command(disable_help_flag = true)]
    Install {
        /// Reinstall without prompting when xbattery is already installed.
        #[arg(short, long)]
        force: bool,
    },
    /// Internal elevated install retry.
    #[command(hide = true, disable_help_flag = true)]
    InstallElevated,
    /// Remove per-user startup task; installed files are left in place.
    #[command(disable_help_flag = true)]
    Uninstall,
    /// Print startup task and installed file status.
    #[command(disable_help_flag = true)]
    Status,
    /// Check GitHub Releases for a newer xbattery version.
    #[command(disable_help_flag = true)]
    CheckUpdate,
    /// Update the installed xbattery executable from GitHub Releases.
    #[command(disable_help_flag = true)]
    Update {
        /// Check what would be updated without changing installed files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Use GameInput events first, with polling fallback.
    #[command(disable_help_flag = true)]
    Monitor,
    /// Print XInput and Windows.Gaming.Input battery reports once.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    Probe,
    /// Test GameInput device callback enumeration.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    GameinputProbe,
    /// Test persistent GameInput callback events for 10 seconds.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    GameinputWatch,
    /// Send the critical battery rumble pattern to the single connected controller.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    RumbleTest,
    /// Test medium, low, and empty battery rumble signal patterns.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    RumbleTestThresholds,
    /// Send a test toast notification.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    ToastTest,
    /// Send a high-priority test toast notification.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    ToastTestHigh,
    /// Send an urgent test toast notification.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    ToastTestUrgent,
    /// Preview production controller notification variants.
    #[cfg(debug_assertions)]
    #[command(disable_help_flag = true)]
    NotificationPreview,
}

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

fn argv(args: impl IntoIterator<Item = String>) -> Vec<String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let args = match args.as_slice() {
        #[cfg(debug_assertions)]
        [arg] if arg == "--probe" || arg == "--once" => vec!["probe".to_owned()],
        #[cfg(debug_assertions)]
        [arg] if arg == "--toast-test" => vec!["toast-test".to_owned()],
        _ => args,
    };

    std::iter::once("xbattery".to_owned()).chain(args).collect()
}

fn print_help(command: Option<&str>) -> AppResult<()> {
    let mut cli = Cli::command();

    match command {
        Some(command) => {
            let Some(subcommand) = cli.find_subcommand(command) else {
                return Err(format!("unknown command `{command}`").into());
            };

            let mut subcommand = subcommand.clone().bin_name(format!("xbattery {command}"));
            subcommand.print_help()?;
        }
        None => cli.print_help()?,
    }

    println!();
    Ok(())
}
