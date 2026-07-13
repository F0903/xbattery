use std::{
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

use xbattery::{
    AppResult,
    controller::backend::{GameInputBackend, WinRTBackend, XInputBackend},
};

pub(super) fn probe() -> AppResult<()> {
    let xinput = XInputBackend;
    let win_rt = WinRTBackend;

    println!("XInput controllers:");
    for report in xinput.diagnostic_reports()? {
        match (report.packet_number, report.battery) {
            (Some(packet_number), Some(battery)) => println!(
                "  slot {}: connected, packet {}, battery {}",
                report.slot + 1,
                packet_number,
                battery.description()
            ),
            _ => println!("  slot {}: disconnected", report.slot + 1),
        }
    }

    println!();
    println!("Windows.Gaming.Input gamepads:");
    let gamepad_reports = win_rt.gamepad_reports()?;
    if gamepad_reports.is_empty() {
        println!("  none");
    }

    for report in gamepad_reports {
        println!("  {}", report.description());
        println!(
            "    remaining_mwh={:?}, full_charge_mwh={:?}",
            report.remaining_mwh, report.full_charge_mwh
        );
    }

    println!();
    println!("Windows.Gaming.Input raw controllers:");
    let raw_reports = win_rt.raw_controller_reports()?;
    if raw_reports.is_empty() {
        println!("  none");
    }

    for report in raw_reports {
        println!("  {}", report.description());
        println!(
            "    remaining_mwh={:?}, full_charge_mwh={:?}",
            report.remaining_mwh, report.full_charge_mwh
        );
    }

    Ok(())
}

pub(super) fn gameinput_probe() -> AppResult<()> {
    let gameinput = GameInputBackend;

    println!("GameInput RegisterDeviceCallback blocking enumeration:");
    let events = gameinput.diagnostic_snapshots()?;

    if events.is_empty() {
        println!("  no gamepad callbacks");
    }

    for (index, event) in events.iter().enumerate() {
        println!(
            "  callback {}: timestamp={}, current=[{}], previous=[{}]",
            index + 1,
            event.timestamp,
            event.current_status,
            event.previous_status
        );
        println!(
            "    battery: {}, status={}, remaining={}, full={}, charge_rate={}",
            event.battery.description(),
            event.battery_status,
            event.remaining_capacity,
            event.full_charge_capacity,
            event.charge_rate
        );
    }

    Ok(())
}

pub(super) fn gameinput_watch() -> AppResult<()> {
    let gameinput = GameInputBackend;

    println!("GameInput persistent callback watcher:");
    let stream = gameinput.start_diagnostic_event_stream()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut count = 0;

    while Instant::now() < deadline {
        let timeout = deadline.saturating_duration_since(Instant::now());

        match stream.recv_timeout(timeout.min(Duration::from_millis(500))) {
            Ok(event) => {
                count += 1;
                println!(
                    "  {} callback {}: timestamp={}, current=[{}], previous=[{}]",
                    event.source,
                    count,
                    event.timestamp,
                    event.current_status,
                    event.previous_status
                );
                println!(
                    "    battery: {}, status={}, remaining={}, full={}, charge_rate={}",
                    event.battery.description(),
                    event.battery_status,
                    event.remaining_capacity,
                    event.full_charge_capacity,
                    event.charge_rate
                );
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err("GameInput callback channel disconnected".into());
            }
        }
    }

    if count == 0 {
        println!("  no callbacks in 10 seconds");
    }

    Ok(())
}
