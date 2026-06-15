use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "xbattery",
    about = "Xbox controller battery notifications",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Command>,
}

#[derive(Subcommand)]
pub(super) enum Command {
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
