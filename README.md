# xbattery

Xbox controller battery notifications for Windows.

xbattery runs quietly in the background and sends Windows toast notifications when a controller connects, disconnects, or drops to a configured battery level.

## Features

- Starts automatically when you sign in.
- Sends battery warnings at 70%, 40%, and 10% by default.
- Lets each configured battery warning level request high-priority or urgent Windows toasts.
- Lets you disable connect/disconnect notifications.
- Works with Xbox Wireless Adapter controllers through Windows controller APIs.

Some Windows controller APIs only expose coarse battery levels instead of exact percentages. In that case xbattery warns when the controller drops through `medium`, `low`, and `empty`.

## Install

Download or build `xbattery.exe`, then run it.

```powershell
.\xbattery.exe
```

Running the app with no command installs it for the current Windows user. If Windows denies access while creating the startup task, xbattery retries through a UAC prompt.

It copies itself to:

```text
%LOCALAPPDATA%\Programs\xbattery
```

It also creates a per-user Scheduled Task named `xbattery` and starts the background monitor. If xbattery is already installed, it asks before overwriting the installed executable and startup task. Existing config is preserved.

Useful commands:

```powershell
xbattery.exe install
xbattery.exe install --force
xbattery.exe status
xbattery.exe check-update
xbattery.exe update
xbattery.exe update --dry-run
xbattery.exe uninstall
```

`uninstall` removes the startup task but leaves the installed files and config in place.

`check-update` looks for a newer GitHub Release. `update` downloads the matching release asset, stops the background monitor, replaces the installed executable, and restarts the monitor if it was running. Use `update --dry-run` to check what would happen without changing files.

The background monitor also checks for updates once a day by default. It shows a toast when a new version is available. Set `updates.auto_install = true` if you want xbattery to install those updates automatically.

## Configuration

After install, edit:

```text
%LOCALAPPDATA%\Programs\xbattery\xbattery.toml
```

The background monitor watches this file and applies valid changes automatically. If a save leaves the file invalid, xbattery keeps using the last valid config until the file is fixed.

Common options:

```toml
[monitor]
poll_interval_seconds = 60

[battery.levels.full]
threshold_percent = 100
coarse_level = "full"
notify = false
urgent = false

[battery.levels.medium]
threshold_percent = 70
coarse_level = "medium"
notify = true
urgent = false

[battery.levels.low]
threshold_percent = 40
coarse_level = "low"
notify = true
urgent = false

[battery.levels.empty]
threshold_percent = 10
coarse_level = "empty"
notify = true
urgent = true

[notifications]
notify_connected = true
notify_disconnected = true

[updates]
repo_owner = "F0903"
repo_name = "xbattery"
asset_identifier = "xbattery"
bin_path_in_archive = "xbattery.exe"
check_automatically = true
check_interval_hours = 24
auto_install = false
notify_available = true
```

Set `notify_connected` or `notify_disconnected` to `false` if you only want battery warnings.

Each battery level can match precise battery APIs with `threshold_percent`, coarse battery APIs with `coarse_level`, or both. Set `notify = true` for levels that should trigger a battery warning, and `urgent = true` for levels that should request urgent Windows toast delivery.

The default `100` / `70` / `40` / `10` percentages match the bucketed values that Windows exposes for most Xbox-style controllers: full, medium, low, and critical/empty. `full` is defined for completeness but does not notify by default.

## Notification Visibility

Windows still decides whether a toast is shown, hidden, or suppressed during a game. xbattery requests higher priority for battery warnings, and requests urgent delivery for critical warnings, but Windows notification settings can override that.

If warnings do not appear, check:

- Windows Settings > System > Notifications
- Focus Assist / Do Not Disturb
- Notification permissions for `xbattery`

## Build From Source

Requirements:

- Rust
- `nuget.exe` on `PATH`
- Microsoft GameInput runtime

Restore the pinned GameInput package and build:

```powershell
cargo xtask gameinput sync
cargo build --release
```

The release executable is:

```text
target\release\xbattery.exe
```

To build the zip asset used by the self-updater:

```powershell
cargo xtask package-release
```

Upload the generated `target\dist\xbattery-v<version>-x86_64-pc-windows-msvc.zip` file to the matching GitHub Release. The zip must contain `xbattery.exe` at the archive root, which is what `bin_path_in_archive = "xbattery.exe"` points to.

GitHub Actions also publishes this asset automatically when a `v<version>` tag is pushed. The tag version must match `Cargo.toml`.

```powershell
git tag v0.1.0
git push origin v0.1.0
```

To install or update the GameInput runtime from this repo:

```powershell
cargo xtask gameinput redist
```

End users can also install the runtime with Winget from an elevated prompt:

```powershell
winget install --id Microsoft.GameInput --exact --source winget
```

For development diagnostics, use a debug build:

```powershell
cargo run -- probe
cargo run -- gameinput-probe
cargo run -- gameinput-watch
cargo run -- notification-preview
```
