use std::{env, fs, io::Read, time::Duration};

use clap::Parser;
use notify_rust::{Notification, Urgency};

/// Simple application to notify when the battery drops too low.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// When the battery drops below this level, send a warning notification.
    #[arg(short, long)]
    warn_percentage: Option<u32>,
    /// When the battery drops below this level, send an urgent critical notification.
    #[arg(short, long)]
    critical_percentage: u32,
    /// The serial number of the battery to check
    #[arg(short, long)]
    serial: String,
}

const POWER_STATE_FILE: &str = "bn_state";

fn main() {
    let args = Args::parse();

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
    match manager
        .batteries()
        .expect("Unable to read batteries")
        .find(|b| {
            b.as_ref()
                .is_ok_and(|f| f.serial_number().is_some_and(|f| args.serial.eq(f)))
        }) {
        Some(Ok(battery)) => {
            let percentage = (battery.state_of_charge().value * 100.0).floor() as u32;
            fs::write(power_state_path, u32::to_le_bytes(percentage))
                .expect("Failed to write power value to file");

            if battery.time_to_full().is_some() {
                // Charging, skip.
            }

            println!("prev: {:?}, curr: {:?}", prev_value, percentage);

            if percentage <= args.critical_percentage && prev_value > args.critical_percentage {
                Notification::new()
                    .summary("Battery level")
                    .body(format!("Battery level is CRITICAL ({:?}%)", percentage).as_str())
                    .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                    .hint(notify_rust::Hint::SoundName("battery-caution".to_string()))
                    .show()
                    .expect("Failed to show notif");
            } else if args
                .warn_percentage
                .is_some_and(|f| percentage <= f && prev_value > f)
            {
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
    };
}
