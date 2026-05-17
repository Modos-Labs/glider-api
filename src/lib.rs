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
        let mut buf = BytesMut::with_capacity(16);
        buf.put_i16(USBCMD_SETMODE);
        buf.put_i16(mode.clone() as i16);
        buf.put_u8(0x00); // WORKAROUND: Alignment is decoded incorrectly in fw.
        buf.put_i16_le(area.x0);
        buf.put_i16_le(area.y0);
        buf.put_i16_le(area.x1);
        buf.put_i16_le(area.y1);
        buf.put_u16(crc16::State::<crc16::XMODEM>::calculate(&buf));
        let device = self.device.lock().unwrap();
        device.0.write(&buf).to_py_err()?;

        let mut response: [u8; 32] = [0; 32];
        device.0.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(()),
        }
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
        let device = self.device.lock().unwrap();
        device.0.write(&buf).to_py_err()?;

        let mut response: [u8; 16] = [0; 16];
        device.0.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(()),
        }
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
    m.add_class::<Rect>()?;
    m.add_class::<Mode>()?;

    Ok(())
}
