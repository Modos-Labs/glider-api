//! Multi-zone layout — set different modes for different screen regions.
//!
//! Divides the 1600×1200 Glider display into three zones:
//!
//! ```text
//! ┌───────────────────┬────────────────┐
//! │                   │                │
//! │  Left half        │  Top-right     │
//! │  FastMonoNoDither │  AutoNoDither  │
//! │  (terminal /      │  (reading /    │
//! │   drawing)        │   maps)        │
//! │                   ├────────────────┤
//! │                   │  Bottom-right  │
//! │                   │  FastMonoBayer │
//! │                   │  (video /      │
//! │                   │   gaming)      │
//! └───────────────────┴────────────────┘
//! ```
//!
//! ```
//! cargo run --example multi_zone
//! ```

use glider_api::{Display, DisplayConfig, Mode, Rect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DisplayConfig::glider_standard();
    let mid_x = config.width / 2;
    let mid_y = config.height / 2;

    let display = Display::new_with_config(&config).map_err(|e| {
        eprintln!("Could not connect: {e}");
        e
    })?;

    // Left half — terminal, code editor, drawing canvas
    let left = Rect::new(0, 0, mid_x, config.height);
    display.set_mode(&Mode::FastMonoNoDither, &left)?;
    println!("Left zone  ({}×{}px): FastMonoNoDither", left.width(), left.height());

    // Top-right — browser, maps, reading app
    let top_right = Rect::new(mid_x, 0, config.width, mid_y);
    display.set_mode(&Mode::AutoNoDither, &top_right)?;
    println!("Top-right  ({}×{}px): AutoNoDither", top_right.width(), top_right.height());

    // Bottom-right — game or video content
    let bottom_right = Rect::new(mid_x, mid_y, config.width, config.height);
    display.set_mode(&Mode::FastMonoBayer, &bottom_right)?;
    println!("Bottom-right ({}×{}px): FastMonoBayer", bottom_right.width(), bottom_right.height());

    println!("\nDone — three zones configured.");
    Ok(())
}
