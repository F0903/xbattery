use xbattery::{AppResult, config::AppConfig, update};

pub(super) fn check() -> AppResult<()> {
    let config = AppConfig::load()?;
    let report = update::check(&config.updates)?;
    println!("{}", report.summary());
    Ok(())
}

pub(super) fn run(dry_run: bool) -> AppResult<()> {
    let config = AppConfig::load()?;
    let report = update::update(&config.updates, dry_run)?;
    println!("{}", report.summary());
    Ok(())
}
