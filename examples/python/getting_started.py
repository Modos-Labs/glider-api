"""
Getting started with the Glider API.

This script connects to a Modos e-ink display and sets the entire screen to a
fast monochrome mode. Run it first to confirm your setup is working.

Prerequisites:
  pip install .                    # build and install the Python binding
  sudo chmod 0666 /dev/hidraw*     # Linux: grant access to the HID device
                                   # (see README for a permanent udev rule)
"""

from glider_api import Display, DisplayConfig, Mode

config = DisplayConfig.glider_standard()

try:
    display = Display.new_with_config(config)
except TypeError as e:
    print(f"Could not connect to display: {e}")
    print("Check that the Glider is plugged in and you have permission to")
    print("access the HID device (see README for Linux udev setup).")
    raise SystemExit(1)

display.set_mode(Mode.FastMonoNoDither, config.full_screen())
print("Done — the display should now be in fast monochrome mode.")
