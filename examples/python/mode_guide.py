"""
Mode guide — cycle through every display mode on a fixed region.

Run this script with a Glider connected to see how each Mode affects the
display. The script pauses between modes so you can observe the difference.

Tip: watch the display carefully during the transition — the speed and
quality of the refresh tell you a lot about which mode to use for your
content.
"""

import time
from glider_api import Display, DisplayConfig, Mode

PAUSE = 3  # seconds to observe each mode

config = DisplayConfig.glider_standard()

try:
    display = Display.new_with_config(config)
except TypeError as e:
    print(f"Could not connect: {e}")
    raise SystemExit(1)

# Use the top-left quarter of the screen as the demo region.
# Replace with config.full_screen() for a whole-display demo.
region = config.full_screen()

modes = [
    (
        Mode.FastMonoNoDither,
        "FastMonoNoDither",
        "Fastest refresh, hard black/white. Best for terminals and code.",
    ),
    (
        Mode.FastMonoBayer,
        "FastMonoBayer",
        "Fast refresh with Bayer dithering. Best for games and moving content.",
    ),
    (
        Mode.FastMonoBlueNoise,
        "FastMonoBlueNoise",
        "Fast refresh, blue-noise dither. Best for images with gradients.",
    ),
    (
        Mode.FastGrey,
        "FastGrey",
        "4-level greyscale. Slowest refresh but best image quality. Best for reading.",
    ),
    (
        Mode.AutoNoDither,
        "AutoNoDither",
        "Hybrid: 1-bit while updating, greyscale when idle. Best for maps/reading apps.",
    ),
    (
        Mode.AutoErrorDiffusion,
        "AutoErrorDiffusion",
        "Like AutoNoDither with error-diffusion dithering for smoother transitions.",
    ),
]

for mode, name, description in modes:
    print(f"\n{name}")
    print(f"  {description}")
    display.set_mode(mode, region)
    time.sleep(PAUSE)

print("\nDone — all modes demonstrated.")
