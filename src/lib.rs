#![doc = include_str!("../intro.md")]
//!
//! This library has a C interface as well as Python bindings. Currently, all
//! methods return Pyo3's `PyResult`; in the future we plan to add a simplified
//! return type for Rust-exclusive use.
//!
//! # Quick Start (Python)
//! ```python
//! from glider_api import Display, DisplayConfig, Mode
//!
//! config = DisplayConfig.glider_standard()
//! display = Display(config)
//! display.set_mode(Mode.FastMonoNoDither, config.full_screen())
//! ```
//!
//! # Quick Start (Rust)
//! ```rust,no_run
//! use glider_api::{Display, DisplayConfig, Mode};
//! let config = DisplayConfig::glider_standard();
//! let display = Display::new_with_config(&config)?;
//! display.set_mode(&Mode::FastMonoNoDither, &config.full_screen())?;
//! # Ok::<(), pyo3::PyErr>(())
//! ```

use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use hidapi::{HidApi, HidDevice, HidError, HidResult};
use pyo3::{exceptions::PyTypeError, prelude::*};

use std::sync::Mutex;

trait ResultExt<T> {
    fn to_py_err(self) -> PyResult<T>;
}

impl<T> ResultExt<T> for HidResult<T> {
    fn to_py_err(self) -> PyResult<T> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => match e {
                HidError::HidApiError { message } => Err(PyTypeError::new_err(message)),
                _ => Err(PyTypeError::new_err("something went wrong")),
            },
        }
    }
}


const VENDOR_ID: u16 = 0x1209;
const PRODUCT_ID: u16 = 0xae86;

/// Display refresh modes supported by the Modos controller.
///
/// Each mode trades off refresh speed, image quality, and ghosting behaviour
/// differently. Choose based on the type of content displayed in each region.
///
/// Modes that mention "dithering" approximate grey values by alternating black
/// and white pixels; this looks better on e-ink than a hard threshold but adds
/// a slight texture.
///
/// The two `ManualLUT` modes require a custom look-up-table to be uploaded to
/// the firmware first. That upload is not yet supported by this API; avoid
/// these modes until support is added.
#[repr(i16)]
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// 1-bit mode driven by a custom firmware look-up-table (LUT).
    /// LUT upload is not yet supported by this API — do not use.
    ManualLUTNoDither = 0,

    /// 1-bit mode with error-diffusion dithering, driven by a custom LUT.
    /// LUT upload is not yet supported by this API — do not use.
    ManualLUTErrorDiffusion = 1,

    /// Fastest 1-bit mode. All grey values snap to black or white with no
    /// dithering. Best for: terminals, code editors, and UI chrome where
    /// hard edges are preferred over tonal accuracy.
    FastMonoNoDither = 2,

    /// 1-bit mode with Bayer (ordered) dithering to approximate grey values.
    /// Best for: games and fast-moving content where some texture is acceptable.
    FastMonoBayer = 3,

    /// 1-bit mode with blue-noise dithering for a less structured appearance
    /// than Bayer. Best for: images with smooth gradients at fast refresh rates.
    FastMonoBlueNoise = 4,

    /// 4-level greyscale mode. Produces the best image quality but has a
    /// significantly slower refresh rate than all other modes.
    /// Best for: static reading content and photographs.
    FastGrey = 5,

    /// Hybrid mode that switches between 1-bit (fast, while content is
    /// changing) and greyscale (once content settles). No dithering.
    /// Best for: mixed-use regions such as maps and reading apps.
    AutoNoDither = 6,

    /// Like `AutoNoDither` but applies error-diffusion dithering during the
    /// fast 1-bit phase, producing smoother transitions.
    /// Best for: mixed-use regions where image quality matters more than speed.
    AutoErrorDiffusion = 7,
}

const USBCMD_REDRAW: i16 = 0x04;
const USBCMD_SETMODE: i16 = 0x05;

/// A rectangular region of the screen, in pixels.
///
/// The coordinate system has its origin at the top-left corner of the display.
/// `x` increases to the right; `y` increases downward. `(x0, y0)` is the
/// top-left corner of the region and `(x1, y1)` is the bottom-right corner
/// (exclusive).
///
/// Use [`DisplayConfig::full_screen`] to get a `Rect` that covers the entire
/// display without hardcoding dimensions.
#[repr(C)]
#[pyclass(get_all)]
pub struct Rect {
    /// Left edge (pixels from the left of the display).
    pub x0: i16,
    /// Top edge (pixels from the top of the display).
    pub y0: i16,
    /// Right edge (exclusive).
    pub x1: i16,
    /// Bottom edge (exclusive).
    pub y1: i16,
}

#[pymethods]
impl Rect {
    /// Create a rectangle from its four corner coordinates (pixels).
    ///
    /// `(x0, y0)` is the top-left corner; `(x1, y1)` is the bottom-right
    /// corner (exclusive). All values are in pixels from the top-left of
    /// the display.
    #[new]
    pub fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// Width of the rectangle in pixels.
    pub fn width(&self) -> i16 {
        self.x1 - self.x0
    }

    /// Height of the rectangle in pixels.
    pub fn height(&self) -> i16 {
        self.y1 - self.y0
    }
}

/// Configuration describing a specific display panel and its USB device identity.
///
/// `DisplayConfig` separates display geometry and USB identity from the live
/// connection ([`Display`]). Create one once and pass it to
/// [`Display::new_with_config`] to open the connection, then reuse it to build
/// [`Rect`] values (e.g. [`DisplayConfig::full_screen`]).
///
/// # Adding new display types
///
/// Call [`DisplayConfig::new`] with the correct dimensions and the USB
/// VID/PID for the target board. Pre-built presets (like
/// [`DisplayConfig::glider_standard`]) are provided for known hardware.
#[repr(C)]
#[pyclass(get_all)]
pub struct DisplayConfig {
    /// Display width in pixels.
    pub width: i16,
    /// Display height in pixels.
    pub height: i16,
    /// USB vendor ID of the display controller.
    pub vendor_id: u16,
    /// USB product ID of the display controller.
    pub product_id: u16,
}

#[pymethods]
impl DisplayConfig {
    /// Create a `DisplayConfig` for any display panel.
    ///
    /// `width` and `height` are in pixels. `vendor_id` and `product_id` are
    /// the USB identifiers for the display controller board.
    #[new]
    pub fn new(width: i16, height: i16, vendor_id: u16, product_id: u16) -> Self {
        Self { width, height, vendor_id, product_id }
    }

    /// Configuration for the standard Glider display (1600 × 1200 pixels).
    ///
    /// Uses VID `0x1209` and PID `0xae86`, which match the current Glider
    /// board. Use this as the starting point for most projects.
    #[staticmethod]
    pub fn glider_standard() -> Self {
        Self::new(1600, 1200, VENDOR_ID, PRODUCT_ID)
    }

    /// Return a [`Rect`] covering the entire display surface.
    pub fn full_screen(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }
}

/// Wrapper that marks HidDevice as Send.
///
/// Safety: The C hidapi library serializes concurrent access to a device handle
/// internally, so it is safe to send an HidDevice to another thread.
struct SendableDevice(HidDevice);
unsafe impl Send for SendableDevice {}

/// An open connection to a Modos e-ink display controller.
///
/// Obtain an instance via [`Display::new`] (uses the standard Glider VID/PID)
/// or [`Display::new_with_config`] (uses the VID/PID from a [`DisplayConfig`]).
///
/// `Display` is safe to share across threads — all USB commands are serialised
/// internally through a mutex.
#[pyclass(frozen)]
pub struct Display {
    device: Mutex<SendableDevice>,
}

#[pymethods]
impl Display {
    /// Connect to the first Modos display found on USB using the standard
    /// Glider VID/PID (`0x1209` / `0xae86`).
    ///
    /// Raises `TypeError` if no matching device is found or the OS denies
    /// access. On Linux you may need to configure udev permissions — see the
    /// README for details.
    ///
    /// **Note:** Uses `HidApi::new_without_enumerate`, which disables HID
    /// device discovery. If another library in the same process also uses
    /// HidApi with enumeration, the two may conflict.
    #[new]
    pub fn new() -> PyResult<Self> {
        let api = HidApi::new_without_enumerate().to_py_err()?;
        let device = api.open(VENDOR_ID, PRODUCT_ID).to_py_err()?;

        Ok(Self { device: Mutex::new(SendableDevice(device)) })
    }

    /// Connect to the display identified by a [`DisplayConfig`].
    ///
    /// Uses `config.vendor_id` and `config.product_id` to locate the device,
    /// allowing non-standard or future Glider boards to be addressed without
    /// changing calling code. Prefer this over `Display()` for new projects.
    ///
    /// Has the same error and HidApi-enumeration caveats as [`Display::new`].
    #[staticmethod]
    pub fn new_with_config(config: &DisplayConfig) -> PyResult<Self> {
        let api = HidApi::new_without_enumerate().to_py_err()?;
        let device = api.open(config.vendor_id, config.product_id).to_py_err()?;

        Ok(Self { device: Mutex::new(SendableDevice(device)) })
    }

    /// Set the refresh mode for a rectangular region of the display.
    ///
    /// This always triggers an immediate redraw of the region in the new mode.
    /// Pass a [`Rect`] describing the area to update; use
    /// [`DisplayConfig::full_screen`] for a whole-display update.
    ///
    /// Choose a [`Mode`] based on the content type — see the `Mode` docs for
    /// per-variant guidance.
    ///
    /// Raises `TypeError` on USB communication errors or if the firmware
    /// rejects the command.
    pub fn set_mode(&self, mode: &Mode, area: &Rect) -> PyResult<()> {
        let buf = build_set_mode_packet(mode, area);
        let device = self.device.lock().unwrap();
        device.0.write(&buf).to_py_err()?;

        let mut response: [u8; 32] = [0; 32];
        device.0.read_timeout(&mut response, 200).to_py_err()?;
        parse_response(&response)
    }

    /// Force a hard refresh of a rectangular region to remove ghosting.
    ///
    /// E-ink displays can retain faint images of previous content ("ghosting").
    /// `clear_and_redraw` fixes this by flashing the region from full-black to
    /// full-white before rendering the current image, at the cost of a visible
    /// flash. Use it when ghosting becomes distracting, not after every update.
    ///
    /// Raises `TypeError` on USB communication errors or if the firmware
    /// rejects the command.
    pub fn redraw(&self, area: &Rect) -> PyResult<()> {
        let buf = build_redraw_packet(area);
        let device = self.device.lock().unwrap();
        device.0.write(&buf).to_py_err()?;

        let mut response: [u8; 16] = [0; 16];
        device.0.read_timeout(&mut response, 200).to_py_err()?;
        parse_response(&response)
    }
}

fn build_set_mode_packet(mode: &Mode, area: &Rect) -> BytesMut {
    let mut buf = BytesMut::with_capacity(16);
    buf.put_i16(USBCMD_SETMODE);
    buf.put_i16(*mode as i16);
    buf.put_u8(0x00); // WORKAROUND: Alignment is decoded incorrectly in fw.
    buf.put_i16_le(area.x0);
    buf.put_i16_le(area.y0);
    buf.put_i16_le(area.x1);
    buf.put_i16_le(area.y1);
    buf.put_u16(crc16::State::<crc16::XMODEM>::calculate(&buf));
    buf
}

fn build_redraw_packet(area: &Rect) -> BytesMut {
    let mut buf = BytesMut::with_capacity(16);
    buf.put_i16(USBCMD_REDRAW);
    buf.put_i16(0x0000); // Dummy param value
    buf.put_u8(0x00); // WORKAROUND: Alignment is decoded incorrectly in fw.
    buf.put_i16_le(area.x0);
    buf.put_i16_le(area.y0);
    buf.put_i16_le(area.x1);
    buf.put_i16_le(area.y1);
    let chksum = crc16::State::<crc16::XMODEM>::calculate(&buf);
    buf.put_u16(chksum);
    buf
}

fn parse_response(response: &[u8]) -> PyResult<()> {
    match LittleEndian::read_u16(response) {
        0x00 => Err(PyTypeError::new_err("invalid command")),
        0x01 => Err(PyTypeError::new_err("checksum incorrect")),
        _ => Ok(()),
    }
}

// C API

#[doc(hidden)]
#[repr(u16)]
pub enum Response {
    Failure = 0x00,
    Success = 0x55,
}

impl<T, E> From<Result<T, E>> for Response {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(_) => Self::Success,
            Err(_) => Self::Failure,
        }
    }
}

/// Connects to the display and returns a `Display` struct for control.
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn create_display(d: &mut Display) -> Response {
    match Display::new() {
        Ok(display) => {
            *d = display;
            return Response::Success;
        }
        Err(_) => return Response::Failure,
    }
}

/// Sets the mode for a region of the display. Note that this will always
/// force a redraw of the region.
#[doc(hidden)]
#[no_mangle]
#[allow(warnings)]
pub extern "C" fn set_mode(d: Display, mode: Mode, area: Rect) -> Response {
    d.set_mode(&mode, &area).into()
}

/// Force a redraw of the region. This will trigger a "flash" of the area
/// from black to white before setting the image, in order to clear any
/// ghosting.
#[doc(hidden)]
#[no_mangle]
#[allow(warnings)]
pub extern "C" fn redraw(d: Display, area: Rect) -> Response {
    d.redraw(&area).into()
}

#[pymodule]
fn glider_api(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Display>()?;
    m.add_class::<DisplayConfig>()?;
    m.add_class::<Rect>()?;
    m.add_class::<Mode>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Rect ---

    #[test]
    fn rect_new_stores_coordinates() {
        let r = Rect::new(10, 20, 30, 40);
        assert_eq!(r.x0, 10);
        assert_eq!(r.y0, 20);
        assert_eq!(r.x1, 30);
        assert_eq!(r.y1, 40);
    }

    #[test]
    fn rect_width_and_height() {
        let r = Rect::new(100, 200, 400, 700);
        assert_eq!(r.width(), 300);
        assert_eq!(r.height(), 500);
    }

    // --- DisplayConfig ---

    #[test]
    fn display_config_new_stores_all_fields() {
        let c = DisplayConfig::new(800, 600, 0x1234, 0x5678);
        assert_eq!(c.width, 800);
        assert_eq!(c.height, 600);
        assert_eq!(c.vendor_id, 0x1234);
        assert_eq!(c.product_id, 0x5678);
    }

    #[test]
    fn display_config_glider_standard_is_1600x1200_with_known_vid_pid() {
        let c = DisplayConfig::glider_standard();
        assert_eq!(c.width, 1600);
        assert_eq!(c.height, 1200);
        assert_eq!(c.vendor_id, 0x1209);
        assert_eq!(c.product_id, 0xae86);
    }

    #[test]
    fn display_config_full_screen_covers_whole_display() {
        let c = DisplayConfig::new(1600, 1200, 0x1209, 0xae86);
        let r = c.full_screen();
        assert_eq!(r.x0, 0);
        assert_eq!(r.y0, 0);
        assert_eq!(r.x1, 1600);
        assert_eq!(r.y1, 1200);
        assert_eq!(r.width(), 1600);
        assert_eq!(r.height(), 1200);
    }

    // --- Mode repr values (regression guard) ---

    #[test]
    fn mode_repr_values() {
        assert_eq!(Mode::ManualLUTNoDither as i16, 0);
        assert_eq!(Mode::ManualLUTErrorDiffusion as i16, 1);
        assert_eq!(Mode::FastMonoNoDither as i16, 2);
        assert_eq!(Mode::FastMonoBayer as i16, 3);
        assert_eq!(Mode::FastMonoBlueNoise as i16, 4);
        assert_eq!(Mode::FastGrey as i16, 5);
        assert_eq!(Mode::AutoNoDither as i16, 6);
        assert_eq!(Mode::AutoErrorDiffusion as i16, 7);
    }

    // --- Packet layout ---

    #[test]
    fn set_mode_packet_layout() {
        let area = Rect::new(0x0010, 0x0020, 0x0030, 0x0040);
        let buf = build_set_mode_packet(&Mode::FastMonoNoDither, &area);

        // Bytes 0-1: USBCMD_SETMODE (0x0005) big-endian
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x05);
        // Bytes 2-3: Mode::FastMonoNoDither = 2, big-endian
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x02);
        // Byte 4: alignment workaround pad
        assert_eq!(buf[4], 0x00);
        // Bytes 5-6: x0 = 0x0010 little-endian
        assert_eq!(buf[5], 0x10);
        assert_eq!(buf[6], 0x00);
        // Bytes 7-8: y0 = 0x0020 little-endian
        assert_eq!(buf[7], 0x20);
        assert_eq!(buf[8], 0x00);
        // Bytes 9-10: x1 = 0x0030 little-endian
        assert_eq!(buf[9], 0x30);
        assert_eq!(buf[10], 0x00);
        // Bytes 11-12: y1 = 0x0040 little-endian
        assert_eq!(buf[11], 0x40);
        assert_eq!(buf[12], 0x00);
        // Bytes 13-14: CRC (non-zero for a non-empty payload)
        let crc = u16::from_be_bytes([buf[13], buf[14]]);
        assert_ne!(crc, 0);
    }

    #[test]
    fn redraw_packet_layout() {
        let area = Rect::new(0x0010, 0x0020, 0x0030, 0x0040);
        let buf = build_redraw_packet(&area);

        // Bytes 0-1: USBCMD_REDRAW (0x0004) big-endian
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x04);
        // Bytes 2-3: dummy param = 0x0000
        assert_eq!(buf[2], 0x00);
        assert_eq!(buf[3], 0x00);
        // Byte 4: alignment workaround pad
        assert_eq!(buf[4], 0x00);
        // Coordinate bytes match set_mode layout
        assert_eq!(buf[5], 0x10);
        assert_eq!(buf[6], 0x00);
        assert_eq!(buf[7], 0x20);
        assert_eq!(buf[8], 0x00);
        assert_eq!(buf[9], 0x30);
        assert_eq!(buf[10], 0x00);
        assert_eq!(buf[11], 0x40);
        assert_eq!(buf[12], 0x00);
        let crc = u16::from_be_bytes([buf[13], buf[14]]);
        assert_ne!(crc, 0);
    }

    #[test]
    fn crc_is_nonzero_for_nonempty_input() {
        let data = [0x00u8, 0x05, 0x00, 0x02, 0x00];
        let crc = crc16::State::<crc16::XMODEM>::calculate(&data);
        assert_ne!(crc, 0);
    }
}
