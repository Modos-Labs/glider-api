"""
Multi-zone layout — set different modes for different regions of the screen.

E-ink displays with the Modos controller can run different refresh modes
simultaneously on different regions of the screen. This lets you optimise
each area independently: fast and crisp for a terminal, high-quality
greyscale for a document, low-latency binary for a game.

This example divides the 1600×1200 Glider display into three zones:

  ┌────────────────┬────────────────┐
  │                │                │
  │  Left half     │  Top-right     │
  │  FastMonoNoDither  AutoNoDither │
  │  (terminal /   │  (reading /    │
  │   drawing)     │   maps)        │
  │                ├────────────────┤
  │                │  Bottom-right  │
  │                │  FastMonoBayer │
  │                │  (gaming)      │
  └────────────────┴────────────────┘

Adjust the Rect coordinates to match your actual layout.
"""

from glider_api import Display, DisplayConfig, Mode, Rect

config = DisplayConfig.glider_standard()
mid_x = config.width // 2
mid_y = config.height // 2

try:
    display = Display.new_with_config(config)
except TypeError as e:
    print(f"Could not connect: {e}")
    raise SystemExit(1)

# Left half — terminal, code editor, drawing canvas
left = Rect(0, 0, mid_x, config.height)
display.set_mode(Mode.FastMonoNoDither, left)
print(f"Left zone  ({left.width()}×{left.height()}px): FastMonoNoDither")

# Top-right — browser, maps, reading app
top_right = Rect(mid_x, 0, config.width, mid_y)
display.set_mode(Mode.AutoNoDither, top_right)
print(f"Top-right  ({top_right.width()}×{top_right.height()}px): AutoNoDither")

# Bottom-right — game or video content
bottom_right = Rect(mid_x, mid_y, config.width, config.height)
display.set_mode(Mode.FastMonoBayer, bottom_right)
print(f"Bottom-right ({bottom_right.width()}×{bottom_right.height()}px): FastMonoBayer")

print("\nDone — three zones configured.")
