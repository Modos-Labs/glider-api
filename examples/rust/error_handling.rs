//! Error handling — how to catch and interpret Glider API errors.
//!
//! The API returns `pyo3::PyErr` for all error conditions. This example
//! shows the known error scenarios and how to handle each one.
//!
//! ```
//! cargo run --example error_handling
//! ```

use glider_api::{Display, DisplayConfig, Mode};

fn main() {
    let config = DisplayConfig::glider_standard();

    // --- Connection errors ---

    println!("Attempting to connect...");
    let display = match Display::new_with_config(&config) {
        Ok(d) => {
            println!("Connected successfully.");
            d
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("no such device") || msg.contains("failed") {
                eprintln!("Device not found: {e}");
                eprintln!("Steps to fix:");
                eprintln!("  1. Confirm the Glider is plugged in via USB.");
                eprintln!("  2. On Linux: add a udev rule (see README) or run");
                eprintln!("       sudo chmod 0666 /dev/hidraw*");
                eprintln!("  3. Confirm VID/PID with: lsusb | grep 1209");
            } else {
                eprintln!("Unexpected connection error: {e}");
            }
            std::process::exit(1);
        }
    };

    // --- Command errors ---

    println!("Setting display mode...");
    match display.set_mode(&Mode::FastMonoNoDither, &config.full_screen()) {
        Ok(()) => {
            println!("Mode set successfully.");
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("checksum") {
                eprintln!("Checksum error (transient): {e}");
                eprintln!("Retrying...");
                display
                    .set_mode(&Mode::FastMonoNoDither, &config.full_screen())
                    .expect("retry failed");
                println!("Retry succeeded.");
            } else if msg.contains("rejected") {
                eprintln!("Firmware rejected command: {e}");
                eprintln!("This may indicate a firmware version mismatch.");
                std::process::exit(1);
            } else {
                eprintln!("Unexpected error during set_mode: {e}");
                std::process::exit(1);
            }
        }
    }
}
