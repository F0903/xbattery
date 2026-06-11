#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{Duration, Instant},
};

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use xbattery::{
    AppResult,
    config::AppConfig,
    controller::service::ControllerService,
    gameinput,
    notifier::{NotificationUrgency, ToastNotifier},
    toast::Toast,
    winrt_input, xinput,
};

fn main() -> AppResult<()> {
    init_com()?;

    match env::args().nth(1).as_deref() {
        Some("probe") | Some("--probe") | Some("--once") => probe(),
        Some("gameinput-probe") => gameinput_probe(),
        Some("gameinput-watch") => gameinput_watch(),
        Some("rumble-test") => rumble_test(),
        Some("rumble-test-thresholds") => rumble_test_thresholds(),
        Some("toast-test") | Some("--toast-test") => {
            let config = AppConfig::load()?;
            Toast::with_config(
                config.toast_config(),
                "xbattery",
                "Toast notifications are working.",
            )
            .send()
        }
        Some("toast-test-high") => {
            let config = AppConfig::load()?;
            Toast::with_config_and_urgency(
                config.toast_config(),
                "xbattery",
                "High priority toast notifications are working.",
                NotificationUrgency::High,
            )
            .send()
        }
        Some("toast-test-urgent") => {
            let config = AppConfig::load()?;
            Toast::with_config_and_urgency(
                config.toast_config(),
                "xbattery",
                "Urgent toast notifications are working.",
                NotificationUrgency::Urgent,
            )
            .send()
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("monitor") | None => {
            let config = AppConfig::load()?;
            let mut service = ControllerService::new(
                ToastNotifier::new(config.toast_config()),
                config.controller_service_config(),
            );
            service.run_until_ctrl_c()
        }
        Some(command) => Err(format!("unknown command: {}", command).into()),
    }
}

fn init_com() -> AppResult<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}

fn rumble_test() -> AppResult<()> {
    let config = AppConfig::load()?;
    let slot = xbattery::controller::rumble::rumble_single_xinput_controller(
        config.rumble.controller_rumble_config(),
        3,
    )?;

    println!("Sent 3 rumble pulses to XInput slot {}.", slot + 1);
    Ok(())
}

fn rumble_test_thresholds() -> AppResult<()> {
    const BETWEEN_PATTERNS: Duration = Duration::from_millis(1500);
    let patterns = [("50% / medium", 1), ("25% / low", 2), ("10% / empty", 3)];
    let config = AppConfig::load()?;
    let rumble_config = config.rumble.controller_rumble_config();
    let slot = xinput::single_connected_slot()?
        .ok_or("rumble-test-thresholds requires exactly one connected XInput controller")?;

    println!(
        "Testing battery threshold rumble patterns on XInput slot {}.",
        slot + 1
    );

    for (index, (label, pulses)) in patterns.iter().enumerate() {
        println!(
            "  {}: {} pulse{}",
            label,
            pulses,
            if *pulses == 1 { "" } else { "s" }
        );
        xbattery::controller::rumble::rumble_xinput_slot(slot, rumble_config.clone(), *pulses)?;

        if index + 1 < patterns.len() {
            thread::sleep(BETWEEN_PATTERNS);
        }
    }

    Ok(())
}

fn probe() -> AppResult<()> {
    println!("XInput controllers:");
    for (index, snapshot) in xinput::poll_controllers()?.iter().enumerate() {
        match snapshot {
            Some(snapshot) => println!(
                "  slot {}: connected, packet {}, battery {}",
                index + 1,
                snapshot.packet_number,
                snapshot.battery.description()
            ),
            None => println!("  slot {}: disconnected", index + 1),
        }
    }

    println!();
    println!("Windows.Gaming.Input raw controllers:");
    let raw_reports = winrt_input::raw_controller_reports()?;
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

fn gameinput_probe() -> AppResult<()> {
    println!("GameInput RegisterDeviceCallback blocking enumeration:");
    let events = gameinput::enumerate_gamepad_snapshots()?;

    if events.is_empty() {
        println!("  no gamepad callbacks");
    }

    for (index, event) in events.iter().enumerate() {
        println!(
            "  callback {}: timestamp={}, current=[{}], previous=[{}]",
            index + 1,
            event.timestamp,
            event.current_status_description(),
            event.previous_status_description()
        );
        println!(
            "    battery: {}, status={}, remaining={}, full={}, charge_rate={}",
            event.battery.description(),
            event.battery_status_description(),
            event.raw_battery.remaining_capacity,
            event.raw_battery.full_charge_capacity,
            event.raw_battery.charge_rate
        );
    }

    Ok(())
}

fn gameinput_watch() -> AppResult<()> {
    println!("GameInput persistent callback watcher:");
    let (_watcher, receiver) = gameinput::start_callback_watcher()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut count = 0;

    while Instant::now() < deadline {
        let timeout = deadline.saturating_duration_since(Instant::now());

        match receiver.recv_timeout(timeout.min(Duration::from_millis(500))) {
            Ok(event) => {
                count += 1;
                let source = event.source_label();
                let event = event.into_snapshot();
                println!(
                    "  {} callback {}: timestamp={}, current=[{}], previous=[{}]",
                    source,
                    count,
                    event.timestamp,
                    event.current_status_description(),
                    event.previous_status_description()
                );
                println!(
                    "    battery: {}, status={}, remaining={}, full={}, charge_rate={}",
                    event.battery.description(),
                    event.battery_status_description(),
                    event.raw_battery.remaining_capacity,
                    event.raw_battery.full_charge_capacity,
                    event.raw_battery.charge_rate
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

fn print_help() {
    println!("xbattery");
    println!();
    println!("Commands:");
    println!("  monitor      Use GameInput events first, with polling fallback");
    println!("  probe        Print XInput and Windows.Gaming.Input battery reports once");
    println!("  gameinput-probe");
    println!("               Test GameInput device callback enumeration");
    println!("  gameinput-watch");
    println!("               Test persistent GameInput callback events for 10 seconds");
    println!("  rumble-test");
    println!("               Send 3 rumble pulses to the single connected XInput controller");
    println!("  rumble-test-thresholds");
    println!("               Test 50%, 25%, and 10% battery rumble patterns");
    println!("  toast-test   Send a test toast notification");
    println!("  toast-test-high");
    println!("               Send a high-priority test toast notification");
    println!("  toast-test-urgent");
    println!("               Send an urgent test toast notification");
}
