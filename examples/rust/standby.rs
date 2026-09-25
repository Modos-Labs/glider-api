//! Standby — put the display into low-power standby and wake it again.
//!
//! The display blanks the panel and stops updating while in standby.
//! Call `exit_standby` to resume normal operation.
//!
//! ```
//! cargo run --example standby
//! ```

use std::{thread, time::Duration};

use glider_api::{Display, DisplayConfig};

const STANDBY_SECS: u64 = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DisplayConfig::glider_standard();

    let display = Display::new_with_config(&config).map_err(|e| {
        eprintln!("Could not connect: {e}");
        e
    })?;

    println!("Entering standby...");
    display.enter_standby()?;
    println!("Display is in standby. Waiting {STANDBY_SECS} seconds...");

    thread::sleep(Duration::from_secs(STANDBY_SECS));

    println!("Exiting standby...");
    display.exit_standby()?;
    println!("Done — display resumed.");

    Ok(())
}
