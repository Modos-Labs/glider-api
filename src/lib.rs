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

const VENDOR_ID: u16 = 0x1209;
const PRODUCT_ID: u16 = 0xae86;

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

const REPORT_ID_CONTROL: u8 = 5;
const USBCMD_REDRAW: i16 = 0x04;
const USBCMD_SETMODE: i16 = 0x05;
const USBCMD_RECV: u8 = 0x08;

// HID packet size the firmware expects. Data chunks fill all but the first
// byte (report ID), giving 63 bytes of payload per write.
const PACKET_SIZE: usize = 64;
const DATA_CHUNK_SIZE: usize = PACKET_SIZE - 1;

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
    /// Define a rectangular area from the four provided coordinates.
    #[new]
    pub fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

/// Core structure defining the display and possible interactions.
#[pyclass(frozen)]
pub struct Display {
    device: HidDevice,
}

unsafe impl Send for Display {}
unsafe impl Sync for Display {}

#[pymethods]
impl Display {
    /// Connects to the display and returns a `Display` struct for control.
    ///
    /// NOTE: This disables device discovery in the HidApi crate. If you are using another
    /// that also uses HidApi, this may lead to conflicts.
    #[new]
    pub fn new() -> PyResult<Self> {
        let api = HidApi::new_without_enumerate().to_py_err()?;
        let device = api.open(VENDOR_ID, PRODUCT_ID).to_py_err()?;

        Ok(Self { device })
    }

    /// Sets the mode for a region of the display. Note that this will always
    /// force a redraw of the region.
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
        self.device.write(&buf).to_py_err()?;

        let mut response: [u8; 32] = [0; 32];
        self.device.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(()),
        }
    }

    /// Force a redraw of the region. This will trigger a "flash" of the area
    /// from black to white before setting the image, in order to clear any
    /// ghosting.
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
        self.device.write(&buf).to_py_err()?;

        let mut response: [u8; 16] = [0; 16];
        self.device.read_timeout(&mut response, 200).to_py_err()?;
        match LittleEndian::read_u16(&response) {
            0x00 => Err(PyTypeError::new_err("invalid command")),
            0x01 => Err(PyTypeError::new_err("checksum incorrect")),
            _ => Ok(()),
        }
    }

    /// Upload a waveform file to the display controller's flash storage.
    ///
    /// Waveform files calibrate the e-ink panel's driving voltages and timing.
    /// Uploading a panel-specific waveform is required before using the
    /// `ManualLUTNoDither` and `ManualLUTErrorDiffusion` modes.
    ///
    /// `filename` is the destination path on the controller's SPIFFS filesystem
    /// (e.g. `"/waveform.wbf"`). `data` is the raw waveform file content.
    ///
    /// The transfer sends the file in 63-byte HID chunks. Large files may take
    /// several seconds. The waveform persists across reboots.
    ///
    /// Use the waveform tools in the firmware repository's `utils/` directory
    /// to convert vendor-supplied `.wbf` files into the expected format.
    pub fn upload_waveform(&self, filename: &str, data: &[u8]) -> PyResult<()> {
        let file_size = data.len() as u32;
        let file_crc = crc16::State::<crc16::XMODEM>::calculate(data) as u32;
        let filename_bytes = filename.as_bytes();

        // Announce the transfer: tell the firmware the filename length, total
        // file size, and expected CRC. Size and CRC are split across the x/y
        // coordinate fields as two LE u16 halves each.
        let cmd = build_recv_packet(filename_bytes.len() as u16, file_size, file_crc);
        self.device.write(&cmd).to_py_err()?;
        let mut response = [0u8; 32];
        self.device.read_timeout(&mut response, 200).to_py_err()?;
        parse_recv_response(&response)?;

        // Stream filename, then file data, in 63-byte raw chunks.
        send_raw_chunks(&self.device, filename_bytes).to_py_err()?;
        send_raw_chunks(&self.device, data).to_py_err()?;

        Ok(())
    }
}

// USBCMD_RECV command packet layout (matches utils/flash_tool/main.py):
//   [0]     REPORT_ID_CONTROL
//   [1]     USBCMD_RECV (u8)
//   [2-3]   filename_len (LE u16)
//   [4-5]   file_size low 16 bits (LE u16)
//   [6-7]   file_size high 16 bits (LE u16)
//   [8-9]   file_crc low 16 bits (LE u16)
//   [10-11] file_crc high 16 bits (LE u16)
//   [12-13] ID = 0 (LE u16)
//   [14-15] packet CRC16-XMODEM over buf[1..14] (LE u16)
//   [16-63] zero padding
fn build_recv_packet(filename_len: u16, file_size: u32, file_crc: u32) -> [u8; PACKET_SIZE] {
    let mut buf = BytesMut::with_capacity(PACKET_SIZE);
    buf.put_u8(REPORT_ID_CONTROL);
    buf.put_u8(USBCMD_RECV);
    buf.put_u16_le(filename_len);
    buf.put_u16_le((file_size & 0xFFFF) as u16);
    buf.put_u16_le((file_size >> 16) as u16);
    buf.put_u16_le((file_crc & 0xFFFF) as u16);
    buf.put_u16_le((file_crc >> 16) as u16);
    buf.put_u16_le(0x0000); // ID
    let pkt_crc = crc16::State::<crc16::XMODEM>::calculate(&buf[1..]);
    buf.put_u16_le(pkt_crc);
    buf.resize(PACKET_SIZE, 0u8);
    buf.as_ref().try_into().unwrap()
}

// Send arbitrary bytes to the device in DATA_CHUNK_SIZE (63-byte) pieces.
// Each write is a 64-byte packet: [REPORT_ID_CONTROL | data... | zero-padding].
// No per-chunk response is read; the firmware buffers data and writes to flash
// when the buffer fills or the transfer completes.
fn send_raw_chunks(device: &HidDevice, data: &[u8]) -> HidResult<()> {
    for chunk in data.chunks(DATA_CHUNK_SIZE) {
        let mut pkt = [0u8; PACKET_SIZE];
        pkt[0] = REPORT_ID_CONTROL;
        pkt[1..1 + chunk.len()].copy_from_slice(chunk);
        device.write(&pkt)?;
    }
    Ok(())
}

// Parse the USBCMD_RECV acknowledgement. Response byte 0 is REPORT_ID_CONTROL;
// the USBRET_* return value is at byte 1.
fn parse_recv_response(response: &[u8]) -> PyResult<()> {
    match response[1] {
        0x00 => Err(PyTypeError::new_err("waveform upload rejected by firmware")),
        0x01 => Err(PyTypeError::new_err("waveform upload: checksum incorrect")),
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
    m.add_class::<Rect>()?;
    m.add_class::<Mode>()?;

    Ok(())
}
