"""
Error handling — how to catch and interpret Glider API errors.

The Glider API currently raises TypeError for all error conditions. This
example shows the known error scenarios and how to handle each one. Future
SDK versions may introduce more specific exception types.

Common errors and their causes:

  "No such device" / "hid_open failed" — device not found or permission denied
      → Check the USB cable, confirm VID/PID match, check udev rules on Linux

  "invalid command" — the firmware rejected the command
      → Unlikely with the standard API; may indicate a firmware version mismatch

  "checksum incorrect" — the packet CRC failed validation on the firmware side
      → Usually transient; retry the command
"""

from glider_api import Display, DisplayConfig, Mode

config = DisplayConfig.glider_standard()

# --- Connection errors ---

print("Attempting to connect...")
try:
    display = Display.new_with_config(config)
    print("Connected successfully.")
except TypeError as e:
    msg = str(e).lower()
    if "no such device" in msg or "hid_open" in msg or "failed" in msg:
        print(f"Device not found: {e}")
        print("Steps to fix:")
        print("  1. Confirm the Glider is plugged in via USB.")
        print("  2. On Linux: run  sudo chmod 0666 /dev/hidraw*")
        print("     or add a permanent udev rule (see README).")
        print("  3. Confirm VID/PID match with  lsusb | grep 1209")
    else:
        print(f"Unexpected connection error: {e}")
    raise SystemExit(1)

# --- Command errors ---

print("Setting display mode...")
try:
    display.set_mode(Mode.FastMonoNoDither, config.full_screen())
    print("Mode set successfully.")
except TypeError as e:
    msg = str(e)
    if "checksum" in msg:
        print(f"Checksum error (transient): {e}")
        print("Retrying...")
        display.set_mode(Mode.FastMonoNoDither, config.full_screen())
        print("Retry succeeded.")
    elif "invalid command" in msg:
        print(f"Firmware rejected command: {e}")
        print("This may indicate a firmware version mismatch.")
    else:
        print(f"Unexpected error during set_mode: {e}")
        raise
