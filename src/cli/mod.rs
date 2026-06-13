#[cfg(debug_assertions)]
mod diagnostics;
mod install;
mod monitor;
#[cfg(debug_assertions)]
mod rumble;
#[cfg(debug_assertions)]
mod toast;
mod update;

use clap::{Parser, Subcommand};
use xbattery::AppResult;

#[derive(Parser)]
#[command(name = "xbattery", about = "Xbox controller battery notifications")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Install per-user startup task and start background monitor.
    Install {
        /// Reinstall without prompting when xbattery is already installed.
        #[arg(short, long)]
        force: bool,
    },
    /// Remove per-user startup task; installed files are left in place.
    Uninstall,
    /// Print startup task and installed file status.
    Status,
    /// Check GitHub Releases for a newer xbattery version.
    CheckUpdate,
    /// Update the installed xbattery executable from GitHub Releases.
    Update {
        /// Check what would be updated without changing installed files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Use GameInput events first, with polling fallback.
    Monitor,
    /// Print XInput and Windows.Gaming.Input battery reports once.
    #[cfg(debug_assertions)]
    Probe,
    /// Test GameInput device callback enumeration.
    #[cfg(debug_assertions)]
    GameinputProbe,
    /// Test persistent GameInput callback events for 10 seconds.
    #[cfg(debug_assertions)]
    GameinputWatch,
    /// Send the critical battery rumble pattern to the single connected controller.
    #[cfg(debug_assertions)]
    RumbleTest,
    /// Test 50%, 25%, and 10% battery rumble signal patterns.
    #[cfg(debug_assertions)]
    RumbleTestThresholds,
    /// Send a test toast notification.
    #[cfg(debug_assertions)]
    ToastTest,
    /// Send a high-priority test toast notification.
    #[cfg(debug_assertions)]
    ToastTestHigh,
    /// Send an urgent test toast notification.
    #[cfg(debug_assertions)]
    ToastTestUrgent,
    /// Preview production controller notification variants.
    #[cfg(debug_assertions)]
    NotificationPreview,
}

pub fn run(args: impl IntoIterator<Item = String>) -> AppResult<()> {
    let cli = Cli::parse_from(argv(args));

    match cli.command {
        None => install::install_interactive(),
        Some(Command::Install { force }) => install::install(false, force),
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
        [arg] if arg == "help" => vec!["--help".to_owned()],
        #[cfg(debug_assertions)]
        [arg] if arg == "--probe" || arg == "--once" => vec!["probe".to_owned()],
        #[cfg(debug_assertions)]
        [arg] if arg == "--toast-test" => vec!["toast-test".to_owned()],
        _ => args,
    };

    std::iter::once("xbattery".to_owned()).chain(args).collect()
}
