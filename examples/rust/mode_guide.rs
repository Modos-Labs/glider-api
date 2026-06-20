//! Mode guide — cycle through every display mode on the full screen.
//!
//! Run this with a Glider connected to see how each Mode affects the display.
//! The program pauses between modes so you can observe the difference.
//!
//! ```
//! cargo run --example mode_guide
//! ```

use std::{thread, time::Duration};

use glider_api::{Display, DisplayConfig, Mode};

const PAUSE: Duration = Duration::from_secs(3);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DisplayConfig::glider_standard();

    let display = Display::new_with_config(&config).map_err(|e| {
        eprintln!("Could not connect: {e}");
        e
    })?;

    let region = config.full_screen();

    let modes: &[(Mode, &str, &str)] = &[
        (Mode::FastMonoNoDither,   "FastMonoNoDither",   "Fastest refresh, hard black/white. Best for terminals and code."),
        (Mode::FastMonoBayer,      "FastMonoBayer",      "Fast refresh with Bayer dithering. Best for games and moving content."),
        (Mode::FastMonoBlueNoise,  "FastMonoBlueNoise",  "Fast refresh, blue-noise dither. Best for images with gradients."),
        (Mode::FastGrey,           "FastGrey",           "4-level greyscale. Slowest refresh, best image quality. Best for reading."),
        (Mode::AutoNoDither,       "AutoNoDither",       "Hybrid: 1-bit while updating, greyscale when idle. Best for maps/reading apps."),
        (Mode::AutoErrorDiffusion, "AutoErrorDiffusion", "Like AutoNoDither with error-diffusion dithering for smoother transitions."),
    ];

    for (mode, name, description) in modes {
        println!("\n{name}");
        println!("  {description}");
        display.set_mode(mode, &region)?;
        thread::sleep(PAUSE);
    }

    println!("\nDone — all modes demonstrated.");
    Ok(())
}
