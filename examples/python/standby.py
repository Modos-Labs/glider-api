"""
Standby — put the display into low-power standby and wake it again.

The display blanks the panel and stops updating while in standby.
Call exit_standby() to resume normal operation.

Run with a Glider connected to a device running firmware v??? or later.

    python examples/python/standby.py
"""

import time
from glider_api import Display, DisplayConfig

STANDBY_SECS = 3

config = DisplayConfig.glider_standard()

try:
    display = Display.new_with_config(config)
except TypeError as e:
    print(f"Could not connect: {e}")
    raise SystemExit(1)

print("Entering standby...")
display.enter_standby()
print(f"Display is in standby. Waiting {STANDBY_SECS} seconds...")

time.sleep(STANDBY_SECS)

print("Exiting standby...")
display.exit_standby()
print("Done — display resumed.")
