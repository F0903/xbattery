# xbattery

Lightweight Windows background service for Xbox controller battery notifications.

## Current shape

- Native Rust app, no .NET runtime and no GC.
- Uses GameInput device and reading callbacks as the primary event path.
- Samples battery on GameInput events. On the tested adapter, that event-triggered sample falls back to XInput because GameInput reports no usable battery capacity.
- Falls back to a configurable poll loop for controller connect/disconnect and battery changes if the GameInput callback watcher cannot run.
- Falls back to XInput battery readings when GameInput sees the controller but does not expose usable battery capacity data.
- Falls back to XInput controller discovery when GameInput does not return connected gamepads.
- Uses `Windows.Gaming.Input` raw controller battery reports as a later precise-percentage fallback when available.
- Sends WinRT toast notifications for:
  - controller connected
  - controller disconnected
  - precise battery crossing configured thresholds
  - coarse XInput battery dropping to `medium`, `low`, or `empty`
- Marks non-critical battery warnings as high priority.
- Marks configured critical precise battery warnings and `empty` battery warnings as urgent to request breakthrough behavior during Focus Assist / Do Not Disturb.
- Optionally sends short XInput rumble pulse patterns when battery warnings fire.
- Includes a `Windows.Gaming.Input.RawGameController` probe to check whether a controller exposes battery capacity values that can be converted to percentages.

## Source layout

- `controller/mod.rs` defines the shared `Controller` domain object and its source.
- `controller/service.rs` owns the background run loop, choosing GameInput callbacks first and polling only as fallback.
- `controller/monitor.rs` tracks previous controller state and emits `ControllerEvent` values.
- `controller/poller.rs` composes the GameInput, XInput, and WinRT polling providers.
- `controller/factory.rs` maps provider-specific snapshots into `Controller` values.
- `controller/event.rs` converts domain events into notification content.
- `controller/rumble.rs` maps battery warning events to optional XInput rumble feedback.
- `battery/mod.rs` defines shared battery readings, kinds, and levels.
- `battery/warning.rs` owns threshold and coarse-level warning policy.
- `config/mod.rs` loads and validates `xbattery.toml`.
- `notifier.rs` abstracts notification delivery; `toast.rs` is the WinRT toast implementation.
- `gameinput/` contains the GameInput public snapshots/events and the isolated raw FFI/callback implementation.

## Commands

```powershell
cargo xtask gameinput sync
cargo run -- probe
cargo run -- gameinput-probe
cargo run -- gameinput-watch
cargo run -- rumble-test
cargo run -- rumble-test-thresholds
cargo run -- toast-test
cargo run -- toast-test-high
cargo run -- toast-test-urgent
cargo run -- monitor
```

`probe` prints both XInput slots and `Windows.Gaming.Input` raw controller battery reports once.

`gameinput-probe` tests Microsoft's newer GameInput `RegisterDeviceCallback` path with a blocking gamepad enumeration and prints any GameInput battery state returned by the device.

`gameinput-watch` starts the persistent GameInput device/input callback watcher and prints callback events for 10 seconds.

`rumble-test` sends three short pulses to the single connected XInput controller. `rumble-test-thresholds` tests the configured 50%, 25%, and 10% pulse patterns with a short pause between each pattern.

`toast-test-high` sends a high-priority toast, and `toast-test-urgent` sends an urgent toast. Urgent toasts can break through Focus Assist / Do Not Disturb only when Windows allows this app to send urgent notifications.

`monitor` starts the GameInput callback watcher first. While that watcher is active, the primary path is event-driven. If it cannot be registered or its event channel fails, the app falls back to the configured polling loop. Press `Ctrl+C` to stop it in debug builds.

Release builds use the Windows subsystem and run without a console window:

```powershell
cargo build --release
.\target\release\xbattery.exe
```

## Configuration

Runtime settings live in `xbattery.toml`. The app checks `XBATTERY_CONFIG` first, then `xbattery.toml` in the current working directory, then `xbattery.toml` next to the executable. If no file exists, built-in defaults are used.

```toml
[monitor]
poll_interval_seconds = 60
control_wait_slice_millis = 250

[battery]
precise_warning_thresholds = [50, 25, 10]

[notifications]
app_id = "xbattery"
notify_connected = true
notify_disconnected = true
urgent_precise_threshold_percent = 10

[rumble]
enabled = false
motor_strength_percent = 35
pulse_millis = 120
gap_millis = 100
```

`precise_warning_thresholds` controls exact-percentage battery warnings. `urgent_precise_threshold_percent` controls when precise warnings become urgent; XInput `empty` is always urgent because it is the lowest coarse level.

Set `notify_connected` or `notify_disconnected` to `false` to suppress those toasts while still tracking controller state for battery warnings.

Set `rumble.enabled` to `true` to add controller rumble feedback for battery warnings. Precise warnings above 25% use one pulse, 25% and below use two pulses, and 10% and below use three pulses. XInput coarse warnings map `medium`/`low`/`empty` to one/two/three pulses.

## GameInput package

The Rust build links against the pinned `Microsoft.GameInput` NuGet package instead of the Windows SDK `GameInput.lib`. The NuGet package is still needed for the headers and native static library; the installed runtime only contains runtime binaries.

Install `nuget.exe` from the NuGet CLI downloads page and make sure it is on `PATH`. Then restore the package declared in `packages.config`:

```powershell
cargo xtask gameinput sync
```

The xtask runs `nuget.exe restore packages.config -PackagesDirectory packages`. The package cache is restored into `packages/`, which is ignored by git.

To update the pinned GameInput package to the latest version published on NuGet:

```powershell
cargo xtask gameinput update
```

To pin a specific version:

```powershell
cargo xtask gameinput pin 3.4.218
```

For the PC runtime, install the redistributable bundled with the pinned NuGet package:

```powershell
cargo xtask gameinput redist
```

This keeps the build-time package and runtime installer on the same GameInput version. The redist task uses `tools/run-elevated.ps1` as a small Windows elevation helper for `msiexec.exe`, so it can prompt for admin rights when needed.

End users can also install or update the GameInput redistributable with Winget from an elevated prompt:

```powershell
winget install --id Microsoft.GameInput --exact --source winget
```

Winget only provisions the runtime/redist. It does not provide the headers or native static library needed to build this project.

## API notes

GameInput is Microsoft's newer input API. This app uses the pinned `Microsoft.GameInput` NuGet native library and calls `GameInputInitialize` for the v0 interface currently mapped in Rust. It then uses `IGameInput::RegisterDeviceCallback` with a blocking initial gamepad enumeration and keeps both device and reading callbacks registered. That is the primary path for controller connect/disconnect and event-triggered battery checks.

On the tested Xbox Wireless Adapter setup, GameInput successfully detects the connected controller, but `IGameInputDevice::GetBatteryState` reports `not-present` with zero capacity values. The monitor therefore keeps the GameInput controller identity and uses XInput as a battery fallback when the GameInput battery state is unusable.

XInput exposes only coarse battery levels:

- full
- medium, treated as roughly 50%
- low, treated as roughly 25%
- empty, treated as roughly 10%

`Windows.Gaming.Input.RawGameController.TryGetBatteryReport` is still probed separately because it can expose capacity values that convert to exact percentages for some devices. On the tested adapter controller it currently returns no raw controllers.

`IGameInput::RegisterReadingCallback` is used to refresh battery state on input events. That makes the primary GameInput route event-driven, but it also means an idle controller that crosses a battery threshold may not notify until the next GameInput device/input event. The XInput/WinRT polling loop is only the fallback when GameInput callbacks are unavailable. XInput does not expose controller connection events.

Rumble uses XInput output. For an XInput-discovered controller, the app can target the exact slot. For GameInput-discovered controllers using XInput battery fallback, rumble is only sent when exactly one XInput controller is connected, so the app does not rumble the wrong controller in multi-controller setups.

## Notification priority

The app uses regular toasts for connect/disconnect notifications. It sets `ToastNotification.Priority = High` for non-critical precise warnings and XInput `medium`/`low` warnings. It also adds `scenario="urgent"` for configured critical precise warnings and XInput `empty` warnings.

Windows still owns final delivery policy. Urgent toasts request permission to break through Focus Assist / Do Not Disturb, but the user can allow or block urgent notifications per app in Windows notification settings.
