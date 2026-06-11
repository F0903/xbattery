use std::io::{self, Write};

use xbattery::{
    AppResult, dialog,
    startup::{StartupInstaller, StartupStatus},
};

pub(super) fn install_interactive() -> AppResult<()> {
    match install(true, false) {
        Ok(()) => Ok(()),
        Err(error) => {
            dialog::show_error("xbattery", &format!("Install failed:\n\n{error}"));
            Err(error)
        }
    }
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

    let report = installer.install(true, overwrite)?;
    println!("{}", report.summary());

    if show_dialog {
        dialog::show_info("xbattery", &report.summary());
    }

    Ok(())
}

pub(super) fn uninstall(show_dialog: bool) -> AppResult<()> {
    let report = StartupInstaller::new()?.uninstall()?;
    println!("{}", report.summary());

    if show_dialog {
        dialog::show_info("xbattery", &report.summary());
    }

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
