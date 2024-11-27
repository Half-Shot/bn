use std::{env, fs, io::Read, path::PathBuf, time::Duration};

use battery::Batteries;
use clap::{value_parser, Arg, Command};
use notify_rust::{Notification, Urgency};

const POWER_STATE_FILE: &str = "bn_state";

fn check_battery(
    mut batteries: Batteries,
    serial: &String,
    prev_value: u32,
    power_state_path: PathBuf,
    critical_percentage: &u32,
    warn_percentage: Option<&u32>,
) {
    match batteries.find(|b| {
        b.as_ref()
            .is_ok_and(|f| f.serial_number().is_some_and(|f| serial.eq(f)))
    }) {
        Some(Ok(battery)) => {
            let percentage = (battery.state_of_charge().value * 100.0).floor() as u32;
            fs::write(power_state_path, u32::to_le_bytes(percentage))
                .expect("Failed to write power value to file");

            if battery.time_to_full().is_some() {
                // Charging, skip.
            }

            println!("prev: {:?}, curr: {:?}", prev_value, percentage);

            if percentage <= *critical_percentage && prev_value > *critical_percentage {
                Notification::new()
                    .summary("Battery level")
                    .body(format!("Battery level is CRITICAL ({:?}%)", percentage).as_str())
                    .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                    .hint(notify_rust::Hint::SoundName("battery-caution".to_string()))
                    .show()
                    .expect("Failed to show notif");
            } else if warn_percentage.is_some_and(|f| percentage <= *f && prev_value > *f) {
                Notification::new()
                    .summary("Battery level")
                    .body(format!("Battery level is low ({:?}%)", percentage).as_str())
                    .hint(notify_rust::Hint::Urgency(Urgency::Normal))
                    .timeout(Duration::from_secs(30))
                    .show()
                    .expect("Failed to show notif");
            }
        }
        Some(Err(e)) => {
            eprintln!("Unable to access battery information {:?}", e);
        }
        None => {
            eprintln!("Unable to find any batteries");
        }
    }
}

fn main() {
    let args = Command::new("bn").about("Simple application to notify when the battery drops too low.").arg(
        Arg::new("warn_percentage")
            .short('w')
            .help("When the battery drops below this level, send a warning notification.")
            .value_parser(value_parser!(u32))
    ).arg(
        Arg::new("critical_percentage")
            .short('c')
            .help("When the battery drops below this level, send an urgent critical notification.")
            .value_parser(value_parser!(u32))
    ).arg(
        Arg::new("serial")
            .short('s')
            .help("The serial number of the battery to check. If not provided, this command will list all batteries.")
            .requires("critical_percentage")
    ).get_matches();

    let power_state_path = env::temp_dir().as_path().join(POWER_STATE_FILE);

    let prev_value = fs::File::open(power_state_path.as_path())
        .and_then(|mut o| {
            let buf = &mut [0; 4];
            o.read_exact(buf)
                .expect("Unknown value in power state file");
            Ok(u32::from_le_bytes(*buf))
        })
        .unwrap_or(100);

    let manager = battery::Manager::new().expect("Unable to start battery manager");
    let batteries = manager.batteries().expect("Unable to read batteries");

    if let Some(battery_serial) = args.get_one::<String>("serial") {
        check_battery(
            batteries,
            battery_serial,
            prev_value,
            power_state_path,
            args.get_one::<u32>("critical_percentage").expect("oh no"),
            args.get_one::<u32>("warn_percentage"),
        );
    } else {
        println!("No serial given, listing possible batteries");
        for possible_battery in batteries {
            if let Ok(battery) = possible_battery {
                println!(
                    " - {:?} (vendor: {:?})",
                    battery.serial_number().or(Some("no id")).unwrap(),
                    battery.vendor().or(Some("no vendor")).unwrap()
                );
            }
        }
    }
}
