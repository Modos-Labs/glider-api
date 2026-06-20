# Glider API Integration Guide

This guide covers the concepts you need to build reliable applications on top
of the Glider SDK. For initial setup, see [GETTING_STARTED.md](GETTING_STARTED.md).

---

## Coordinate system

The display coordinate system has its **origin at the top-left corner**.
`x` increases to the right; `y` increases downward.

```
(0, 0) ──────────────────────────► x
  │
  │          Glider display
  │          1600 × 1200 px
  │
  ▼
  y                        (1600, 1200)
```

A `Rect(x0, y0, x1, y1)` describes the top-left corner `(x0, y0)` and the
bottom-right corner `(x1, y1)` (exclusive). All values are in **pixels**.

Use `DisplayConfig` to avoid hardcoding dimensions:

```python
config = DisplayConfig.glider_standard()  # width=1600, height=1200
full   = config.full_screen()             # Rect(0, 0, 1600, 1200)
half_l = Rect(0, 0, config.width // 2, config.height)
```

---

## Choosing a display mode

| Content type | Recommended mode | Why |
|---|---|---|
| Terminal, code editor, UI | `FastMonoNoDither` | Fastest refresh; hard edges look correct for text |
| Games, fast-moving content | `FastMonoBayer` | Fast with light dithering for smoother motion |
| Images with gradients | `FastMonoBlueNoise` | Less structured dither pattern than Bayer |
| Static reading, documents, photos | `FastGrey` | 4-level greyscale; best quality, slowest refresh |
| Maps, reading apps (mixed) | `AutoNoDither` | Fast during updates, greyscale when idle |
| Mixed content, quality preferred | `AutoErrorDiffusion` | Like AutoNoDither with smoother dithering |
| Custom LUT | `ManualLUTNoDither` / `ManualLUTErrorDiffusion` | **Not yet supported** — requires firmware LUT upload |

`set_mode` always triggers a redraw of the region in the new mode. You do not
need to call `redraw` after `set_mode`.

---

## Multi-zone layouts

The Modos controller applies modes per-region, not globally. You can call
`set_mode` with different `Rect` values to run different modes simultaneously
on different parts of the screen.

```python
config  = DisplayConfig.glider_standard()
display = Display.new_with_config(config)
mid_x   = config.width // 2
mid_y   = config.height // 2

display.set_mode(Mode.FastMonoNoDither, Rect(0,     0,     mid_x,        config.height))
display.set_mode(Mode.AutoNoDither,     Rect(mid_x, 0,     config.width, mid_y))
display.set_mode(Mode.FastMonoBayer,    Rect(mid_x, mid_y, config.width, config.height))
```

Zones can overlap — the last `set_mode` call wins for any pixel in the
overlap area.

---

## Clearing ghosting with `redraw`

E-ink displays retain a faint ghost of previous content even after new
content is drawn. The `redraw` method forces a hard refresh: it flashes the
region from full-black to full-white before rendering the current image.

```python
display.redraw(config.full_screen())  # clears the whole screen
```

Use `redraw` sparingly — the flash is visible and disruptive. Typical
situations where it helps:

- After switching from high-contrast content (dark background) to light content
- When ghosting has built up over many partial updates
- Before a presentation or demo where a clean slate matters

You do **not** need to call `redraw` after every `set_mode`; `set_mode`
already redraws the region in the new mode.

---

## Error handling

All errors currently raise `TypeError`. Inspect the message to determine
the cause:

```python
try:
    display = Display.new_with_config(config)
    display.set_mode(Mode.FastMonoNoDither, config.full_screen())
except TypeError as e:
    msg = str(e).lower()
    if "no such device" in msg or "hid_open" in msg:
        ...  # device not found or no permission
    elif "checksum" in msg:
        ...  # transient CRC error — safe to retry
    elif "invalid command" in msg:
        ...  # firmware rejected the command
```

See [examples/python/error_handling.py](examples/python/error_handling.py)
for a complete example.

---

## Thread safety

`Display` is safe to use from multiple threads. USB commands are serialised
internally through a mutex, so you can call `set_mode` and `redraw` from
different threads without external locking.

```python
import threading

def update_left(display, config):
    display.set_mode(Mode.FastMonoNoDither, Rect(0, 0, config.width // 2, config.height))

def update_right(display, config):
    display.set_mode(Mode.AutoNoDither, Rect(config.width // 2, 0, config.width, config.height))

config  = DisplayConfig.glider_standard()
display = Display.new_with_config(config)

t1 = threading.Thread(target=update_left,  args=(display, config))
t2 = threading.Thread(target=update_right, args=(display, config))
t1.start(); t2.start()
t1.join();  t2.join()
```

---

## Known limitations

| Limitation | Detail |
|---|---|
| ManualLUT modes not supported | `ManualLUTNoDither` and `ManualLUTErrorDiffusion` require uploading a custom LUT to firmware. This is not yet implemented in the SDK. |
| HidApi enumeration conflict | `Display` uses `HidApi::new_without_enumerate`, which disables device discovery. If another library in the same process also uses HidApi with enumeration enabled, they may conflict. |
| Linux udev required for non-root | Access to `/dev/hidraw*` requires either `sudo chmod 0666` or a permanent udev rule. See [GETTING_STARTED.md](GETTING_STARTED.md). |
| Single device per process | The SDK does not yet support connecting to multiple Glider units simultaneously. |
| No pip / crates.io release | The package must be built from source. `pip install .` from the repo root is the supported install path. |
