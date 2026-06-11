use std::{
    env, fs,
    path::{Path, PathBuf},
};

const GAMEINPUT_PACKAGE_ID: &str = "Microsoft.GameInput";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=packages.config");
    println!("cargo:rerun-if-changed=nuget.config");
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let package_version = gameinput_package_version(&manifest_dir);
    println!(
        "cargo:rerun-if-changed=packages/Microsoft.GameInput.{}/native/lib",
        package_version
    );

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let lib_arch = match target_arch.as_str() {
        "aarch64" => "arm64",
        _ => "x64",
    };
    let lib_dir = manifest_dir
        .join("packages")
        .join(format!("Microsoft.GameInput.{}", package_version))
        .join("native")
        .join("lib")
        .join(lib_arch);

    if !lib_dir.join("GameInput.lib").exists() {
        panic!(
            "Microsoft.GameInput {} was not found. Run `cargo xtask gameinput sync`.",
            package_version
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=GameInput");
}

fn gameinput_package_version(manifest_dir: &Path) -> String {
    let packages_config = manifest_dir.join("packages.config");
    let text = fs::read_to_string(&packages_config)
        .unwrap_or_else(|err| panic!("failed to read {}: {}", packages_config.display(), err));

    for line in text.lines() {
        if line.contains("id=\"Microsoft.GameInput\"")
            && let Some(version) = attribute_value(line, "version")
        {
            return version.to_string();
        }
    }

    panic!(
        "{} must contain {}",
        packages_config.display(),
        GAMEINPUT_PACKAGE_ID
    );
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{}=\"", name);
    let start = line.find(&prefix)? + prefix.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}
