//! Redraw — force a hard refresh to clear ghosting.
//!
//! ```
//! cargo run --example redraw
//! ```

use glider_api::{Display, DisplayConfig, Rect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DisplayConfig::glider_standard();
    let display = Display::new_with_config(&config)?;

    println!("Redrawing top-left 200×200 region...");
    display.redraw(&Rect::new(0, 0, 200, 200))?;
    println!("OK");

    println!("Redrawing full screen...");
    display.redraw(&config.full_screen())?;
    println!("OK");

    Ok(())
}
