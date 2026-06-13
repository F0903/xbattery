use std::{
    fs::{self, File},
    io,
    path::Path,
    process::{Command, Stdio},
};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{
    Result,
    process::{ensure_exists, repo_root, run},
};

const EXE_NAME: &str = "xbattery.exe";

pub fn package() -> Result<()> {
    let root = repo_root()?;
    run(Command::new("cargo").arg("build").arg("--release"))?;

    let version = package_version(&root.join("Cargo.toml"))?;
    let target = rustc_host_target()?;
    let exe = root.join("target").join("release").join(EXE_NAME);
    ensure_exists(&exe, "release executable")?;

    let dist = root.join("target").join("dist");
    fs::create_dir_all(&dist)?;

    let zip = dist.join(format!("xbattery-v{version}-{target}.zip"));
    write_release_zip(&zip, &exe)?;

    println!("Release package: {}", zip.display());
    println!("Upload this zip as a GitHub Release asset.");

    Ok(())
}

fn write_release_zip(zip_path: &Path, exe_path: &Path) -> Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut exe = File::open(exe_path)?;

    zip.start_file(EXE_NAME, options)?;
    io::copy(&mut exe, &mut zip)?;
    zip.finish()?;

    Ok(())
}

fn package_version(cargo_toml: &Path) -> Result<String> {
    let content = fs::read_to_string(cargo_toml)?;

    for line in content.lines() {
        let line = line.trim();
        if let Some(version) = line
            .strip_prefix("version = \"")
            .and_then(|rest| rest.strip_suffix('"'))
        {
            return Ok(version.to_string());
        }
    }

    Err(format!("could not find package version in {}", cargo_toml.display()).into())
}

fn rustc_host_target() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!("rustc -vV failed with status {}", output.status).into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        if let Some(host) = line.strip_prefix("host: ") {
            return Ok(host.to_string());
        }
    }

    Err("could not parse host target from rustc -vV".into())
}
