from glider_api import Display, DisplayConfig, Rect

config = DisplayConfig.glider_standard()
d = Display.new_with_config(config)

print("Redrawing top-left 200x200 region...")
d.redraw(Rect(0, 0, 200, 200))
print("OK")

print("Redrawing full screen...")
d.redraw(config.full_screen())
print("OK")