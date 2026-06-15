use clap::CommandFactory;
use xbattery::AppResult;

use super::command::Cli;

pub(super) fn print_help(command: Option<&str>) -> AppResult<()> {
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
