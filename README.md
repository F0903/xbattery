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

Here is a small piano-roll-style warning sound:

```toml
[battery.levels.low]
threshold_percent = 40
coarse_level = "low"
notify = true

generated_sound.roll = [
    { notes = "C5 E5 G5", at = 0.0, length = 0.20, gain = 0.18 },
    { notes = "G5 B5 D6", at = 0.22, length = 0.24, gain = 0.16 },
]
generated_sound.effects = [
    { kind = "reverb", room_seconds = 0.20, damping = 0.50, mix = 0.10 },
    { kind = "soft_limiter", drive = 1.1 },
]
```

See the bundled `xbattery.toml` for a complete configuration.

Set `notify_connected` or `notify_disconnected` to `false` if you only want battery warnings.

Each battery level can match precise battery APIs with `threshold_percent`, coarse battery APIs with `coarse_level`, or both. Set `notify = true` for levels that should show a battery warning toast, and `urgent = true` for levels that should request urgent Windows toast delivery.

Use `generated_sound.roll` to have xbattery build an in-memory audio clip when the monitor configuration is applied, then play that clip when the level is reached. Each entry is a piano-roll event: `notes` is a whitespace-separated note or chord, while `at` and `length` position it in seconds. `gain` controls its volume. Events can overlap and use `sine`, `triangle`, `square`, or `sawtooth` waves; sine is the default, and another wave can be selected with `wave = "triangle"`.

For envelope control, add `adsr = [attack, decay, sustain, release]`. Attack, decay, and release are measured in seconds; sustain is a level from 0 to 1. Omitting `adsr` uses the built-in envelope. The generated clip can also be shaped with `low_pass`, `delay`, `reverb`, and `soft_limiter` effects.

The previous `generated_sound.layers` notation has been removed. Its `start_seconds`, `duration_seconds`, `volume`, `waveform`, and individual envelope fields are now `at`, `length`, `gain`, `wave`, and `adsr` inside `generated_sound.roll`.

Notes use case-insensitive scientific pitch notation with A4 tuned to 440 Hz and C4 as middle C. Sharps and flats are supported, such as `F#4` and `Bb4`, across the range C-1 through G9. For fine detuning, add up to 100 cents in either direction, such as `C5+9.07c` or `A4-12c`. The detuned pitch must remain inside the supported range and below half of `sample_rate`.

For simpler sequential sounds, `generated_sound.segments` supports `tone` and `silence` entries. Tone segments use `notes`; silence segments omit them. A generated sound must define either `roll` or `segments`, not both.

Set `sound_file` instead if you want to point a level at your own `.wav` file. App-side audio works independently of toast display, so a level with `notify = false` and `generated_sound` or `sound_file` can play audio without showing a Windows notification.

For example, a short sequential melody can be written as:

```toml
[battery.levels.low]
threshold_percent = 40
coarse_level = "low"

generated_sound.segments = [
    { kind = "tone", notes = "C5 E5 G5", duration_seconds = 0.15 },
    { kind = "silence", duration_seconds = 0.05 },
    { kind = "tone", notes = "G5 B5 D6", duration_seconds = 0.22 },
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
