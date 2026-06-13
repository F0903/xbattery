use std::{env, process::Command};

mod gameinput;
mod process;
mod release;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("gameinput") => match args.next().as_deref() {
            Some("sync") | None => gameinput::sync(None),
            Some("update") => gameinput::update(),
            Some("pin") => gameinput::pin(
                args.next()
                    .ok_or("usage: cargo xtask gameinput pin <version>")?,
            ),
            Some("redist") => gameinput::redist(),
            Some(command) => Err(format!("unknown gameinput command: {command}").into()),
        },
        Some("package-release") => release::package(),
        Some("check") => check(),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

fn check() -> Result<()> {
    process::run(Command::new("cargo").arg("fmt").arg("--check"))?;
    process::run(Command::new("cargo").arg("test"))?;
    process::run(Command::new("cargo").arg("build"))?;
    Ok(())
}

fn print_help() {
    println!("cargo xtask");
    println!();
    println!("Commands:");
    println!("  gameinput sync          Restore pinned Microsoft.GameInput package");
    println!("  gameinput update        Pin latest Microsoft.GameInput package and restore");
    println!("  gameinput pin <version> Pin a specific Microsoft.GameInput version and restore");
    println!(
        "  gameinput redist        Install the pinned GameInput redist through elevation helper"
    );
    println!("  package-release         Build release exe and zip it for GitHub Releases");
    println!("  check                   Run cargo fmt --check, cargo test, and cargo build");
}
