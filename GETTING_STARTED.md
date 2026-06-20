# Getting Started with the Glider API

This guide takes you from a freshly cloned repo to a working command that
changes your Glider's display mode.

---

## Prerequisites

- A **Modos Glider** connected via USB
- **Python 3.12+** and **Rust** installed
  (Rust is needed to compile the Python extension — install via [rustup.rs](https://rustup.rs))
- **Linux only:** `pkg-config` and `libudev-dev`

```sh
# Ubuntu / Debian
sudo apt install pkg-config libudev-dev
```

---

## 1. Install the SDK

Clone the repository and install the Python package:

```sh
git clone <repo-url>
cd glider-api
pip install .
```

This compiles the Rust extension and installs it into your current Python
environment.

---

## 2. Grant USB access (Linux only)

Most Linux systems deny non-root access to HID devices by default.

**Quick fix (resets on reboot):**

```sh
sudo chmod 0666 /dev/hidraw*
```

**Permanent fix:** add a udev rule so access survives reboots:

```sh
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1209", ATTRS{idProduct}=="ae86", MODE="0666"' \
  | sudo tee /etc/udev/rules.d/99-glider.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## 3. Your first command

```python
from glider_api import Display, DisplayConfig, Mode

config = DisplayConfig.glider_standard()   # 1600×1200, standard Glider VID/PID
display = Display.new_with_config(config)
display.set_mode(Mode.FastMonoNoDither, config.full_screen())
```

Run it:

```sh
python examples/python/getting_started.py
```

**What you should see:** the display briefly flashes as it refreshes, then
settles into fast monochrome mode. Text and UI elements will appear sharper;
the display will update faster for subsequent redraws.

---

## 4. What just happened?

| Step | What it does |
|------|-------------|
| `DisplayConfig.glider_standard()` | Describes the standard Glider panel (1600×1200 px, USB VID/PID) |
| `Display.new_with_config(config)` | Opens a USB connection to the device |
| `config.full_screen()` | Returns a `Rect` covering the full 1600×1200 display |
| `display.set_mode(Mode.FastMonoNoDither, ...)` | Sends the mode-change command over USB |

---

## 5. Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `TypeError: No such device` | Device not found or no permission | Check USB cable; run the udev / chmod step above |
| `TypeError: hid_open failed` | Permission denied on Linux | Repeat step 2 |
| Nothing changes on screen | Wrong region coordinates | Use `config.full_screen()` to rule out a Rect issue |

---

## Next steps

- **[Mode guide](examples/python/mode_guide.py)** — try every display mode and see the difference
- **[Multi-zone layout](examples/python/multi_zone.py)** — run different modes on different screen regions simultaneously
- **[Error handling](examples/python/error_handling.py)** — handle connection and command errors robustly
- **[Integration guide](INTEGRATION_GUIDE.md)** — deeper reference for coordinate system, ghosting, thread safety, and known limitations
