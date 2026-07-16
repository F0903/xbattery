use std::io::{self, Write};

use crate::{
    AppResult, dialog, elevate,
    startup::{StartupInstaller, StartupStatus, is_startup_access_denied},
};

pub(super) fn install_interactive() -> AppResult<()> {
    match install(true, false) {
        Ok(()) => Ok(()),
        Err(error) if is_startup_access_denied(error.as_ref()) => retry_install_elevated(error),
        Err(error) => {
            dialog::show_error("xbattery", &format!("Install failed:\n\n{error}"));
            Err(error)
        }
    }
}

fn retry_install_elevated(error: Box<dyn std::error::Error + Send + Sync>) -> AppResult<()> {
    match elevate::relaunch_current_exe_as_admin("install-elevated") {
        Ok(()) => Ok(()),
        Err(elevation_error) => {
            let message = format!(
                "Install failed:\n\n{error}\n\nCould not start elevated installer:\n\n{elevation_error}"
            );
            dialog::show_error("xbattery", &message);
            Err(elevation_error)
        }
    }
}

pub(super) fn install_elevated_retry() -> AppResult<()> {
    install(true, true)
}

pub(super) fn install(show_dialog: bool, force: bool) -> AppResult<()> {
    let installer = StartupInstaller::new()?;
    let status = installer.status();
    let already_installed = status.has_install_state();
    let overwrite = if already_installed {
        force || confirm_overwrite(show_dialog, &status)?
    } else {
        false
    };

    if already_installed && !overwrite {
        let message = "Install cancelled.";
        println!("{message}");

        if show_dialog {
            dialog::show_info("xbattery", message);
        }

        return Ok(());
    }

    let report = installer.install(overwrite)?;
    println!("{}", report.summary());

    if show_dialog {
        dialog::show_info("xbattery", &report.summary());
    }

    Ok(())
}

pub(super) fn uninstall() -> AppResult<()> {
    let report = StartupInstaller::new()?.uninstall()?;
    println!("{}", report.summary());

    Ok(())
}

pub(super) fn status() -> AppResult<()> {
    let status = StartupInstaller::new()?.status();
    println!("{}", status.summary());
    Ok(())
}

fn confirm_overwrite(show_dialog: bool, status: &StartupStatus) -> AppResult<bool> {
    let message = format!(
        "xbattery already appears to be installed:\n\n{}\n\nOverwrite the startup task and executable?\n\nExisting config will be preserved.",
        status.summary()
    );

    if show_dialog {
        return Ok(dialog::ask_yes_no("xbattery", &message));
    }

    println!("{message}");
    print!("\nOverwrite? [y/N]: ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    Ok(matches!(
        response.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
