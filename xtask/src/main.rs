use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const GAMEINPUT_PACKAGE_ID: &str = "Microsoft.GameInput";
const NUGET_SOURCE: &str = "https://api.nuget.org/v3/index.json";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("gameinput") => match args.next().as_deref() {
            Some("sync") | None => gameinput_sync(None),
            Some("update") => {
                let latest = latest_gameinput_version()?;
                gameinput_sync(Some(latest))
            }
            Some("pin") => {
                let version = args
                    .next()
                    .ok_or("usage: cargo xtask gameinput pin <version>")?;
                gameinput_sync(Some(version))
            }
            Some("redist") => gameinput_redist(),
            Some(command) => Err(format!("unknown gameinput command: {command}").into()),
        },
        Some("check") => check(),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

fn gameinput_sync(target_version: Option<String>) -> Result<()> {
    let root = repo_root()?;
    let packages_config = root.join("packages.config");
    let current_version = gameinput_version(&packages_config)?;
    let target_version = target_version.unwrap_or(current_version.clone());

    if target_version != current_version {
        set_gameinput_version(&packages_config, &target_version)?;
        println!("Updated {GAMEINPUT_PACKAGE_ID} from {current_version} to {target_version}");
    }

    let nuget = nuget_exe()?;
    run(Command::new(&nuget)
        .arg("restore")
        .arg(&packages_config)
        .arg("-PackagesDirectory")
        .arg(root.join("packages"))
        .arg("-ConfigFile")
        .arg(root.join("nuget.config"))
        .arg("-NonInteractive")
        .arg("-Verbosity")
        .arg("quiet"))?;

    let package_dir = gameinput_package_dir(&root, &target_version);
    let lib = package_dir
        .join("native")
        .join("lib")
        .join("x64")
        .join("GameInput.lib");
    let redist = gameinput_redist_path_for_version(&root, &target_version);

    ensure_exists(&lib, "native library")?;
    ensure_exists(&redist, "redistributable")?;

    println!("Restored {GAMEINPUT_PACKAGE_ID} {target_version}");
    println!("Native lib: {}", lib.display());
    println!("Redist MSI: {}", redist.display());

    Ok(())
}

fn gameinput_redist() -> Result<()> {
    let root = repo_root()?;
    let version = gameinput_version(root.join("packages.config"))?;
    let redist = gameinput_redist_path_for_version(&root, &version);
    let log = root.join("target").join("gameinput-redist-install.log");

    if !redist.exists() {
        gameinput_sync(Some(version.clone()))?;
    }

    ensure_exists(&redist, "redistributable")?;
    fs::create_dir_all(log.parent().ok_or("redist log path has no parent")?)?;

    let helper = root.join("tools").join("run-elevated.ps1");
    ensure_exists(&helper, "elevation helper")?;

    run(Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(helper)
        .arg("-FilePath")
        .arg("msiexec.exe")
        .arg("-ArgumentList")
        .arg(format!(
            "/i \"{}\" /quiet /norestart /L*v \"{}\"",
            redist.display(),
            log.display()
        )))?;

    println!("Started elevated GameInput redist install for {version}");
    println!("Log: {}", log.display());

    Ok(())
}

fn check() -> Result<()> {
    run(Command::new("cargo").arg("fmt").arg("--check"))?;
    run(Command::new("cargo").arg("test"))?;
    run(Command::new("cargo").arg("build"))?;
    Ok(())
}

fn latest_gameinput_version() -> Result<String> {
    let nuget = nuget_exe()?;
    let output = Command::new(nuget)
        .arg("search")
        .arg(GAMEINPUT_PACKAGE_ID)
        .arg("-Source")
        .arg(NUGET_SOURCE)
        .arg("-Take")
        .arg("10")
        .arg("-NonInteractive")
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!("nuget.exe search failed with status {}", output.status).into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        let Some(rest) = line.trim_start().strip_prefix("> Microsoft.GameInput |") else {
            continue;
        };

        let version = rest
            .split('|')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("could not parse Microsoft.GameInput version from nuget output")?;

        return Ok(version.to_string());
    }

    Err("could not find Microsoft.GameInput in nuget search output".into())
}

fn gameinput_version(path: impl AsRef<Path>) -> Result<String> {
    let content = fs::read_to_string(path)?;
    read_attribute(&content, "version")
        .map(str::to_string)
        .ok_or_else(|| "packages.config must contain Microsoft.GameInput version".into())
}

fn set_gameinput_version(path: &Path, version: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let current = read_attribute(&content, "version")
        .ok_or("packages.config must contain Microsoft.GameInput version")?;
    let updated = content.replace(
        &format!("version=\"{current}\""),
        &format!("version=\"{version}\""),
    );

    fs::write(path, updated)?;
    Ok(())
}

fn read_attribute<'a>(content: &'a str, attribute: &str) -> Option<&'a str> {
    let package_start = content.find(&format!("id=\"{GAMEINPUT_PACKAGE_ID}\""))?;
    let content = &content[package_start..];
    let attr_start = content.find(&format!("{attribute}=\""))? + attribute.len() + 2;
    let content = &content[attr_start..];
    let attr_end = content.find('"')?;

    Some(&content[..attr_end])
}

fn gameinput_package_dir(root: &Path, version: &str) -> PathBuf {
    root.join("packages")
        .join(format!("{GAMEINPUT_PACKAGE_ID}.{version}"))
}

fn gameinput_redist_path_for_version(root: &Path, version: &str) -> PathBuf {
    gameinput_package_dir(root, version)
        .join("redist")
        .join("GameInputRedist.msi")
}

fn nuget_exe() -> Result<PathBuf> {
    if let Some(path) = find_on_path("nuget.exe") {
        return Ok(path);
    }

    let fallback = PathBuf::from(r"C:\nuget\nuget.exe");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err("nuget.exe was not found on PATH or at C:\\nuget\\nuget.exe".into())
}

fn find_on_path(file_name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    env::split_paths(&path)
        .map(|path| path.join(file_name))
        .find(|path| path.exists())
}

fn repo_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest directory has no parent")?
        .to_path_buf())
}

fn ensure_exists(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(format!("Expected {label} was not found: {}", path.display()).into())
    }
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed with status {status:?}").into())
    }
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
    println!("  check                   Run cargo fmt --check, cargo test, and cargo build");
}
