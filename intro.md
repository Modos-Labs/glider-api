Common API for interacting with e-ink displays driven by the Modos display
controller, which allows for fast refresh of e-ink displays. 

The Modos display controller includes:
- Support both monochrome and color-filter-array (such as Kaleido(TM)) based color screen
- Extremely low processing delay of <20 us
- Binary and 4-level grayscale output modes
- Latency-optimized binary and 4-level grayscale driving modes
- Hybrid automatic binary and 16-level grayscale driving mode
- Hardware bayer dithering, blue-noise dithering, and error-diffusion dithering with no additional latency

This API enables programmatic control of regional updates and mode switches,
using the USB HID interface exposed by the display controller. Specifically,
You can set any rectangular region of the display to a specific display
mode, or force a refresh of any region.