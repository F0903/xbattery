# xbattery

Xbox controller battery notifications for Windows.

xbattery runs quietly in the background and sends Windows toast notifications when a controller connects, disconnects, or drops to a configured battery level. It can also play short controller rumble signals for low battery warnings.

## Features

- Starts automatically when you sign in.
- Sends battery warnings at 50%, 25%, and 10% by default.
- Uses high-priority or urgent Windows toasts for battery warnings.
- Lets you disable connect/disconnect notifications.
- Supports optional custom rumble patterns for each battery warning stage.
- Works with Xbox Wireless Adapter controllers through Windows controller APIs.

Some Windows controller APIs only expose coarse battery levels instead of exact percentages. In that case xbattery warns when the controller drops through `medium`, `low`, and `empty`.

## Install

Download or build `xbattery.exe`, then run it.

```powershell
.\xbattery.exe
```

Running the app with no command installs it for the current Windows user. It copies itself to:

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

## Configuration

After install, edit:

```text
%LOCALAPPDATA%\Programs\xbattery\xbattery.toml
```

Common options:

```toml
[monitor]
poll_interval_seconds = 60

[battery]
precise_warning_thresholds = [50, 25, 10]

[notifications]
notify_connected = true
notify_disconnected = true
urgent_precise_threshold_percent = 10

[updates]
repo_owner = "F0903"
repo_name = "xbattery"
asset_identifier = "xbattery"
bin_path_in_archive = "xbattery.exe"

[rumble]
enabled = false
gap_millis = 45
group_gap_millis = 200
```

Set `notify_connected` or `notify_disconnected` to `false` if you only want battery warnings.

Set `rumble.enabled` to `true` to add rumble feedback when a battery warning fires.

### Rumble Patterns

Rumble patterns are made from named "jolts", which define the strength and duration of the rumble feedback. You can then use these to define patterns or groups of patterns to play at each battery level.

```toml
[rumble.jolts.quick]
handle_strength_percent = 100
trigger_strength_percent = 75
handle_millis = 35
trigger_millis = 50

[rumble.jolts.strong]
handle_strength_percent = 100
trigger_strength_percent = 100
handle_millis = 75
trigger_millis = 100

[rumble.patterns.medium]
groups = [["quick", "quick"]]

[rumble.patterns.low]
groups = [["quick", "quick", "strong"]]

[rumble.patterns.empty]
groups = [["quick", "quick", "strong"], ["quick", "quick", "strong"]]
```

Each jolt starts in the controller handles and quickly moves into the triggers. Each pattern group runs its jolts in order; `group_gap_millis` separates groups.

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
cargo run -- rumble-test
cargo run -- rumble-test-thresholds
```
