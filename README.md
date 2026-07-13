# xbattery

Xbox controller battery notifications for Windows.

xbattery runs quietly in the background and sends Windows toast notifications, with optional local audio, when a controller connects, disconnects, or drops to a configured battery level.

## Features

- Starts automatically when you sign in.
- Sends battery warnings at 70%, 40%, and 10% by default.
- Lets each configured battery level choose toast priority, urgency, and optional local audio.
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

generated_sound.layers = [
    { waveform = "sine", frequencies = [523.25, 526.0, 659.25], start_seconds = 0.0, duration_seconds = 0.24, volume = 0.155, attack_seconds = 0.018, decay_seconds = 0.120, sustain_level = 0.32, release_seconds = 0.110 },
    { waveform = "sine", frequencies = [261.63, 263.0, 329.63], start_seconds = 0.0, duration_seconds = 0.34, volume = 0.085, attack_seconds = 0.022, decay_seconds = 0.180, sustain_level = 0.38, release_seconds = 0.140 },
    { waveform = "sine", frequencies = [1046.5, 1051.0], start_seconds = 0.018, duration_seconds = 0.18, volume = 0.034, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.18, release_seconds = 0.080 },
    { waveform = "sine", frequencies = [659.25, 662.5, 783.99], start_seconds = 0.205, duration_seconds = 0.26, volume = 0.135, attack_seconds = 0.018, decay_seconds = 0.130, sustain_level = 0.30, release_seconds = 0.120 },
    { waveform = "sine", frequencies = [329.63, 331.0, 392.0], start_seconds = 0.205, duration_seconds = 0.36, volume = 0.075, attack_seconds = 0.024, decay_seconds = 0.190, sustain_level = 0.34, release_seconds = 0.150 },
    { waveform = "sine", frequencies = [1318.51, 1325.0], start_seconds = 0.223, duration_seconds = 0.18, volume = 0.028, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.16, release_seconds = 0.080 },
]
generated_sound.effects = [
    { kind = "low_pass", cutoff_hz = 6800.0 },
    { kind = "delay", delay_seconds = 0.105, feedback = 0.08, mix = 0.045 },
    { kind = "reverb", room_seconds = 0.24, damping = 0.62, mix = 0.10 },
    { kind = "soft_limiter", drive = 1.04 },
]

[battery.levels.low]
threshold_percent = 40
coarse_level = "low"
notify = true
urgent = false

generated_sound.layers = [
    { waveform = "sine", frequencies = [523.25, 526.0, 659.25], start_seconds = 0.0, duration_seconds = 0.24, volume = 0.160, attack_seconds = 0.018, decay_seconds = 0.120, sustain_level = 0.32, release_seconds = 0.110 },
    { waveform = "sine", frequencies = [261.63, 263.0, 329.63], start_seconds = 0.0, duration_seconds = 0.35, volume = 0.090, attack_seconds = 0.022, decay_seconds = 0.185, sustain_level = 0.38, release_seconds = 0.145 },
    { waveform = "sine", frequencies = [1046.5, 1051.0], start_seconds = 0.018, duration_seconds = 0.18, volume = 0.036, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.18, release_seconds = 0.080 },
    { waveform = "sine", frequencies = [659.25, 662.5, 783.99], start_seconds = 0.205, duration_seconds = 0.26, volume = 0.145, attack_seconds = 0.018, decay_seconds = 0.130, sustain_level = 0.30, release_seconds = 0.120 },
    { waveform = "sine", frequencies = [329.63, 331.0, 392.0], start_seconds = 0.205, duration_seconds = 0.37, volume = 0.082, attack_seconds = 0.024, decay_seconds = 0.195, sustain_level = 0.34, release_seconds = 0.155 },
    { waveform = "sine", frequencies = [1318.51, 1325.0], start_seconds = 0.223, duration_seconds = 0.18, volume = 0.030, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.16, release_seconds = 0.080 },
    { waveform = "sine", frequencies = [392.0, 394.0, 493.88], start_seconds = 0.455, duration_seconds = 0.32, volume = 0.150, attack_seconds = 0.020, decay_seconds = 0.160, sustain_level = 0.28, release_seconds = 0.145 },
    { waveform = "sine", frequencies = [196.0, 197.0, 246.94], start_seconds = 0.455, duration_seconds = 0.45, volume = 0.092, attack_seconds = 0.030, decay_seconds = 0.235, sustain_level = 0.34, release_seconds = 0.180 },
    { waveform = "sine", frequencies = [783.99, 790.0], start_seconds = 0.478, duration_seconds = 0.22, volume = 0.032, attack_seconds = 0.014, decay_seconds = 0.095, sustain_level = 0.14, release_seconds = 0.090 },
]
generated_sound.effects = [
    { kind = "low_pass", cutoff_hz = 6800.0 },
    { kind = "delay", delay_seconds = 0.105, feedback = 0.08, mix = 0.050 },
    { kind = "reverb", room_seconds = 0.26, damping = 0.62, mix = 0.11 },
    { kind = "soft_limiter", drive = 1.05 },
]

[battery.levels.empty]
threshold_percent = 10
coarse_level = "empty"
notify = true
urgent = true

generated_sound.layers = [
    { waveform = "sine", frequencies = [987.77, 992.0, 1318.51], start_seconds = 0.0, duration_seconds = 0.15, volume = 0.150, attack_seconds = 0.010, decay_seconds = 0.070, sustain_level = 0.18, release_seconds = 0.070 },
    { waveform = "sine", frequencies = [493.88, 496.0, 659.25], start_seconds = 0.0, duration_seconds = 0.23, volume = 0.092, attack_seconds = 0.016, decay_seconds = 0.120, sustain_level = 0.28, release_seconds = 0.095 },
    { waveform = "sine", frequencies = [1760.0, 1768.0], start_seconds = 0.010, duration_seconds = 0.10, volume = 0.026, attack_seconds = 0.006, decay_seconds = 0.040, sustain_level = 0.08, release_seconds = 0.045 },
    { waveform = "sine", frequencies = [739.99, 744.0, 987.77], start_seconds = 0.175, duration_seconds = 0.16, volume = 0.165, attack_seconds = 0.010, decay_seconds = 0.076, sustain_level = 0.18, release_seconds = 0.074 },
    { waveform = "sine", frequencies = [369.99, 372.0, 493.88], start_seconds = 0.175, duration_seconds = 0.25, volume = 0.100, attack_seconds = 0.017, decay_seconds = 0.130, sustain_level = 0.28, release_seconds = 0.105 },
    { waveform = "sine", frequencies = [1479.98, 1488.0], start_seconds = 0.188, duration_seconds = 0.11, volume = 0.030, attack_seconds = 0.006, decay_seconds = 0.043, sustain_level = 0.08, release_seconds = 0.045 },
    { waveform = "sine", frequencies = [392.0, 394.0, 493.88], start_seconds = 0.390, duration_seconds = 0.24, volume = 0.205, attack_seconds = 0.016, decay_seconds = 0.120, sustain_level = 0.22, release_seconds = 0.110 },
    { waveform = "sine", frequencies = [196.0, 197.0, 246.94], start_seconds = 0.390, duration_seconds = 0.36, volume = 0.132, attack_seconds = 0.024, decay_seconds = 0.190, sustain_level = 0.34, release_seconds = 0.150 },
    { waveform = "sine", frequencies = [783.99, 790.0], start_seconds = 0.410, duration_seconds = 0.16, volume = 0.038, attack_seconds = 0.010, decay_seconds = 0.062, sustain_level = 0.10, release_seconds = 0.065 },
]
generated_sound.effects = [
    { kind = "low_pass", cutoff_hz = 6800.0 },
    { kind = "delay", delay_seconds = 0.072, feedback = 0.05, mix = 0.035 },
    { kind = "reverb", room_seconds = 0.18, damping = 0.62, mix = 0.10 },
    { kind = "soft_limiter", drive = 1.08 },
]

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

Each battery level can match precise battery APIs with `threshold_percent`, coarse battery APIs with `coarse_level`, or both. Set `notify = true` for levels that should show a battery warning toast, and `urgent = true` for levels that should request urgent Windows toast delivery.

Use `generated_sound.layers` to have xbattery build an in-memory audio clip when the monitor configuration is applied, then play that clip when the level is reached. Layers can overlap, use `sine`, `triangle`, `square`, or `sawtooth` waveforms, and can be shaped with attack, decay, sustain, release, `low_pass`, `delay`, `reverb`, and `soft_limiter` controls.

For simpler sequential sounds, `generated_sound.segments` supports `tone` and `silence` entries. A generated sound must define either `layers` or `segments`, not both.

Set `sound_file` instead if you want to point a level at your own `.wav` file. App-side audio works independently of toast display, so a level with `notify = false` and `generated_sound` or `sound_file` can play audio without showing a Windows notification.

For example:

```toml
[battery.levels.low]
threshold_percent = 40
coarse_level = "low"

generated_sound.layers = [
    { waveform = "sine", frequencies = [523.25, 526.0, 659.25], start_seconds = 0.0, duration_seconds = 0.24, volume = 0.160, attack_seconds = 0.018, decay_seconds = 0.120, sustain_level = 0.32, release_seconds = 0.110 },
    { waveform = "sine", frequencies = [261.63, 263.0, 329.63], start_seconds = 0.0, duration_seconds = 0.35, volume = 0.090, attack_seconds = 0.022, decay_seconds = 0.185, sustain_level = 0.38, release_seconds = 0.145 },
    { waveform = "sine", frequencies = [1046.5, 1051.0], start_seconds = 0.018, duration_seconds = 0.18, volume = 0.036, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.18, release_seconds = 0.080 },
    { waveform = "sine", frequencies = [659.25, 662.5, 783.99], start_seconds = 0.205, duration_seconds = 0.26, volume = 0.145, attack_seconds = 0.018, decay_seconds = 0.130, sustain_level = 0.30, release_seconds = 0.120 },
    { waveform = "sine", frequencies = [329.63, 331.0, 392.0], start_seconds = 0.205, duration_seconds = 0.37, volume = 0.082, attack_seconds = 0.024, decay_seconds = 0.195, sustain_level = 0.34, release_seconds = 0.155 },
    { waveform = "sine", frequencies = [1318.51, 1325.0], start_seconds = 0.223, duration_seconds = 0.18, volume = 0.030, attack_seconds = 0.012, decay_seconds = 0.080, sustain_level = 0.16, release_seconds = 0.080 },
    { waveform = "sine", frequencies = [392.0, 394.0, 493.88], start_seconds = 0.455, duration_seconds = 0.32, volume = 0.150, attack_seconds = 0.020, decay_seconds = 0.160, sustain_level = 0.28, release_seconds = 0.145 },
    { waveform = "sine", frequencies = [196.0, 197.0, 246.94], start_seconds = 0.455, duration_seconds = 0.45, volume = 0.092, attack_seconds = 0.030, decay_seconds = 0.235, sustain_level = 0.34, release_seconds = 0.180 },
    { waveform = "sine", frequencies = [783.99, 790.0], start_seconds = 0.478, duration_seconds = 0.22, volume = 0.032, attack_seconds = 0.014, decay_seconds = 0.095, sustain_level = 0.14, release_seconds = 0.090 },
]
generated_sound.effects = [
    { kind = "low_pass", cutoff_hz = 6800.0 },
    { kind = "delay", delay_seconds = 0.105, feedback = 0.08, mix = 0.050 },
    { kind = "reverb", room_seconds = 0.26, damping = 0.62, mix = 0.11 },
    { kind = "soft_limiter", drive = 1.05 },
]
```

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
cargo run -- config-audio-test
cargo run -- notification-preview
```
