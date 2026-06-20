//! Getting started with the Glider API.
//!
//! Connects to a Modos e-ink display and sets the entire screen to fast
//! monochrome mode. Run this first to confirm your setup is working.
//!
//! ```
//! cargo run --example getting_started
//! ```

use glider_api::{Display, DisplayConfig, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DisplayConfig::glider_standard();

    let display = Display::new_with_config(&config).map_err(|e| {
        eprintln!("Could not connect to display: {e}");
        eprintln!("Check that the Glider is plugged in and you have permission");
        eprintln!("to access the HID device (see README for Linux udev setup).");
        e
    })?;

    display.set_mode(&Mode::FastMonoNoDither, &config.full_screen())?;
    println!("Done — the display is now in fast monochrome mode.");
    Ok(())
}
