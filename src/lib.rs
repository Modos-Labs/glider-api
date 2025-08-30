#![doc = include_str!("../intro.md")]
//!
//! This library has a C interface as well as Python bindings. Currently, all
//! methods return Pyo3's `PyResult`; in the future we plan to add a simplified
//! return type for Rust-exclusive use.
//! 
//! Example
//! ```rust
//! use glider_api::{Mode, Rect, Display}
//! let display = Display::new()?;
//! display.set_mode(Mode::FastMonoBlueNoise, Rect{x0: 0, y0: 0, x1: 1000, y1: 1000})?;
//! ```

use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use hidapi::{HidApi, HidDevice, HidError, HidResult};
use pyo3::{exceptions::PyTypeError, prelude::*};

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


impl<T> ResultExt<T> for serialport::Result<T> {
    fn to_py_err(self) -> PyResult<T> {
        match self {
            Ok(x) => Ok(x),
            Err(_e) => Err(PyTypeError::new_err("something went wrong")),
        }
    }
}

const VENDOR_ID: u16 = 0x0483;
const PRODUCT_ID: u16 = 0x5750;

/// Modes supported by the display controller. 
/// 
/// *ManualLUTNoDither*: 1-bit mode with a custom look-up-table (LUT). Note that
/// this API does not support uploading manual LUTs at this time.
///
/// *ManualLUTErrorDiffusion*: 1-bit mode with a custom look-up-table (LUT), 
/// using error diffusion dithering to approximate grey values. Note that this 
/// API does not support uploading manual LUTs at this time.
///
/// *FastMonoNoDither*: 1-bit mode. All gray values are converted to either 
/// black or white.
///
/// *FastMonoBayer*: 1-bit mode with Bayer dithering.
///
/// *FastMonoBlueNoise*: 1-bit mode with dithering based on a blue noise 
/// pattern.
///
/// *FastGrey*: Optimized 4-level grey mode. Note this mode has a much slower 
/// refresh rate compared to all other modes.
///
/// *AutoNoDither*: Optimized display mode that switches between 1-bit and gray 
/// values depending on the speed of update. When the input image is changed, it 
/// switches to 1-bit mode with no dithering and does the update. When the image
/// hasn’t changed for a while, it re-renders the image in greyscale.
///
/// *AutoErrorDiffusion*: Like `AutoNoDither`, but uses error diffusion to 
/// approximate grey values during for image updates.
#[repr(i16)]
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// 1-bit mode with a custom look-up-table (LUT). Note that this API does
    /// not support uploading manual LUTs at this time.
    ManualLUTNoDither = 0,

    /// 1-bit mode with a custom look-up-table (LUT), using error diffusion 
    /// dithering to approximate grey values. Note that this API does not 
    /// support uploading manual LUTs at this time.
    ManualLUTErrorDiffusion = 1,

    /// 1-bit mode. All gray values are converted to either black or white.
    FastMonoNoDither = 2,

    /// 1-bit mode with Bayer dithering.
    FastMonoBayer = 3,

    /// 1-bit mode with dithering based on a blue noise pattern.
    FastMonoBlueNoise = 4,

    /// Optimized 4-level grey mode. Note this mode has a much slower refresh 
    /// rate compared to all other modes.
    FastGrey = 5,

    /// Optimized display mode that switches between 1-bit and gray values
    /// depending on the speed of update. When the input image is changed, it 
    /// switches to 1-bit mode with no dithering and does the update. When the 
    /// image hasn’t changed for a while, it re-renders the image in greyscale.
    AutoNoDither = 6,

    /// Like `AutoNoDither`, but uses error diffusion to approximate grey values
    /// during for image updates.
    AutoErrorDiffusion = 7,
}

const USBCMD_REDRAW: i16 = 0x04;
const USBCMD_SETMODE: i16 = 0x05;

#[repr(C)]
#[pyclass]
/// A rectangular area of the screen, used for redrawing as well as setting
/// modes.
pub struct Rect {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

#[pymethods]
impl Rect {
    #[new]
    fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

/// Core structure defining the display and possible interactions.
#[pyclass(frozen)]
pub struct Display {
    device: HidDevice
}

unsafe impl Send for Display {}
unsafe impl Sync for Display {}


#[pymethods]
impl Display {

    /// Connects to the display and returns a `Display` struct for control.
    #[new]
    fn new() -> PyResult<Self> {
        let api = HidApi::new_without_enumerate().to_py_err()?;
        let device = api.open(VENDOR_ID, PRODUCT_ID).to_py_err()?;

        Ok(Self { device })
    }

    /// Sets the mode for a region of the display. Note that this will always
    /// force a redraw of the region.
    fn set_mode(&self, mode: &Mode, area: &Rect) -> PyResult<()> {
        let mut buf = BytesMut::with_capacity(16);
        buf.put_i16(USBCMD_SETMODE);
        buf.put_i16(mode.clone() as i16);
        buf.put_u8(0x00); // WORKAROUND: Alignment is decoded incorrectly in fw. 
        buf.put_i16_le(area.x0);
        buf.put_i16_le(area.y0);
        buf.put_i16_le(area.x1);
        buf.put_i16_le(area.y1);
        buf.put_u16(crc16::State::<crc16::XMODEM>::calculate(&buf));
        self.device.write(&buf).to_py_err()?;

        let mut response: [u8; 32] = [0; 32];
        self.device.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(())
        }
    }

    /// Force a redraw of the region. This will trigger a "flash" of the area
    /// from black to white before setting the image, in order to clear any
    /// ghosting.
    fn redraw(&self, area: &Rect) -> PyResult<()> {
        let mut buf = BytesMut::with_capacity(16);

        buf.put_i16(USBCMD_REDRAW);
        buf.put_i16(0x0000); // Dummy param value
        buf.put_u8(0x00); // WORKAROUND: Alignment is decoded incorrectly in fw. 
        buf.put_i16_le(area.x0);
        buf.put_i16_le(area.y0);
        buf.put_i16_le(area.x1);
        buf.put_i16_le(area.y1);
                 
        let chksum= crc16::State::<crc16::XMODEM>::calculate(&buf);
        buf.put_u16(chksum);
        self.device.write(&buf).to_py_err()?;

        let mut response: [u8; 16] = [0; 16];
        self.device.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(())
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
