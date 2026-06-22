//! Transport layer — discovers CDC port, sends requests, receives responses.
//!
//! Linux-only: opens /dev/ttyACM* directly via raw file I/O.
//! No `serialport` / `libudev` dependency required.

use openhotas_protocol::frame::{crc16_ccitt, FrameParser, MAX_PAYLOAD_SIZE, SOF_A, SOF_B};
use openhotas_protocol::request::Request;
use openhotas_protocol::response::{DeviceInfo, Response};
use openhotas_protocol::version::PROTOCOL_VERSION_MAJOR;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

/// Connection to the OpenHOTAS CDC port.
pub struct OpenHotasTransport {
    file: File,
    parser: FrameParser,
}

/// Errors that can occur during transport operations.
#[derive(Debug)]
pub enum TransportError {
    /// No OpenHOTAS device found (auto-detect).
    DeviceNotFound,
    /// Failed to open the serial port.
    PortError(String),
    /// Sending the request frame failed.
    SendError(String),
    /// No response received within timeout.
    Timeout,
    /// Read error on the serial port.
    ReadError(String),
    /// Response payload could not be deserialized.
    InvalidResponse,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound => write!(
                f,
                "No OpenHOTAS device found. Is it connected? (tried /dev/ttyACM0-9)"
            ),
            Self::PortError(e) => write!(f, "Port error: {e}"),
            Self::SendError(e) => write!(f, "Send error: {e}"),
            Self::Timeout => write!(f, "No response from device (timeout)"),
            Self::ReadError(e) => write!(f, "Read error: {e}"),
            Self::InvalidResponse => write!(f, "Could not parse response"),
        }
    }
}

impl OpenHotasTransport {
    /// Auto-detect and connect to the first CDC port that answers GetInfo.
    ///
    /// Tries /dev/ttyACM0 through /dev/ttyACM9, plus /dev/ttyUSB0-9.
    pub fn connect() -> Result<Self, TransportError> {
        let mut found_port = false;
        for i in 0..10 {
            let path = format!("/dev/ttyACM{i}");
            if let Some(transport) = Self::try_open_openhotas(&path, &mut found_port) {
                return Ok(transport);
            }
        }
        for i in 0..10 {
            let path = format!("/dev/ttyUSB{i}");
            if let Some(transport) = Self::try_open_openhotas(&path, &mut found_port) {
                return Ok(transport);
            }
        }
        if found_port {
            Err(TransportError::PortError(
                "serial ports found, but none answered as OpenHOTAS".into(),
            ))
        } else {
            Err(TransportError::DeviceNotFound)
        }
    }

    /// Connect to a specific port by path.
    pub fn connect_to(path: &str) -> Result<Self, TransportError> {
        let mut transport = Self::open(path)?;
        transport.validate_openhotas()?;
        Ok(transport)
    }

    fn try_open_openhotas(path: &str, found_port: &mut bool) -> Option<Self> {
        if fs::metadata(path).is_err() {
            return None;
        }
        *found_port = true;
        let mut transport = Self::open(path).ok()?;
        transport.validate_openhotas().ok()?;
        Some(transport)
    }

    fn open(path: &str) -> Result<Self, TransportError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(path)
            .map_err(|e| TransportError::PortError(format!("Cannot open {path}: {e}")))?;

        // Configure termios for raw binary CDC communication
        configure_tty(&file)?;

        Ok(Self {
            file,
            parser: FrameParser::new(),
        })
    }

    /// Send a request and wait for the response.
    pub fn send(&mut self, request: Request) -> Result<Response, TransportError> {
        let frame = self.build_request_frame(&request)?;
        self.file
            .write_all(&frame)
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        self.file
            .flush()
            .map_err(|e| TransportError::SendError(e.to_string()))?;

        self.read_response().or_else(|e| {
            if matches!(e, TransportError::Timeout) {
                self.read_response()
            } else {
                Err(e)
            }
        })
    }

    pub fn get_info(&mut self) -> Result<DeviceInfo, TransportError> {
        match self.send(Request::GetInfo)? {
            Response::Info(info) => Ok(info),
            Response::Error(e) => Err(TransportError::PortError(format!(
                "GetInfo rejected by device: {e:?}"
            ))),
            other => Err(TransportError::PortError(format!(
                "unexpected GetInfo response: {other:?}"
            ))),
        }
    }

    fn validate_openhotas(&mut self) -> Result<(), TransportError> {
        let info = self.get_info()?;
        if info.protocol_major != PROTOCOL_VERSION_MAJOR {
            return Err(TransportError::PortError(format!(
                "unsupported protocol major {} (expected {})",
                info.protocol_major, PROTOCOL_VERSION_MAJOR
            )));
        }
        if info.axis_count != 3 || info.button_count != 32 {
            return Err(TransportError::PortError(format!(
                "unexpected device shape: {} axes, {} buttons",
                info.axis_count, info.button_count
            )));
        }
        Ok(())
    }

    /// Build a binary frame for a Request.
    fn build_request_frame(&mut self, request: &Request) -> Result<Vec<u8>, TransportError> {
        let mut payload_buf = [0u8; MAX_PAYLOAD_SIZE];
        let payload = postcard::to_slice(request, &mut payload_buf)
            .map_err(|_| TransportError::SendError("serialization failed".into()))?;
        let payload_len = payload.len();

        let frame_len = 4 + payload_len + 2;
        let mut frame = vec![0u8; frame_len];

        frame[0] = SOF_A;
        frame[1] = SOF_B;
        let len_bytes = (payload_len as u16).to_be_bytes();
        frame[2] = len_bytes[0];
        frame[3] = len_bytes[1];
        frame[4..4 + payload_len].copy_from_slice(payload);

        let crc = crc16_ccitt(&frame[2..4 + payload_len]);
        let crc_bytes = crc.to_be_bytes();
        frame[4 + payload_len] = crc_bytes[0];
        frame[4 + payload_len + 1] = crc_bytes[1];

        Ok(frame)
    }

    /// Read bytes until a complete response frame is received.
    fn read_response(&mut self) -> Result<Response, TransportError> {
        let mut byte_buf = [0u8; 1];
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(TransportError::Timeout);
            }

            match self.file.read(&mut byte_buf) {
                Ok(1) => match self.parser.feed(byte_buf[0]) {
                    Ok(Some(frame)) => {
                        return postcard::from_bytes::<Response>(&frame.payload)
                            .map_err(|_| TransportError::InvalidResponse);
                    }
                    Ok(None) => continue,
                    Err(_) => continue, // frame CRC error — keep scanning
                },
                Ok(0) => {
                    // EOF — device disconnected?
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Ok(_) => continue, // read >1 byte (shouldn't happen with 1-byte buf)
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Non-blocking, no data yet — brief sleep then retry
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => return Err(TransportError::ReadError(e.to_string())),
            }
        }
    }
}

/// Configure the TTY for raw binary CDC communication.
///
/// CDC ACM does not use baud rate, parity, or flow control in the
/// traditional sense — the USB stack handles framing. But we set
/// raw mode to avoid the kernel's line discipline interfering.
fn configure_tty(file: &File) -> Result<(), TransportError> {
    let fd = file.as_raw_fd();

    // Get current attributes
    let mut termios: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(fd, &mut termios) } != 0 {
        return Err(TransportError::PortError("tcgetattr failed".into()));
    }

    // Raw mode: no canonical processing, no echo, no signals
    // cfmakeraw equivalent
    termios.c_iflag &= !(libc::IGNBRK
        | libc::BRKINT
        | libc::PARMRK
        | libc::ISTRIP
        | libc::INLCR
        | libc::IGNCR
        | libc::ICRNL
        | libc::IXON);
    termios.c_oflag &= !libc::OPOST;
    termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
    termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
    termios.c_cflag |= libc::CS8;

    // VMIN = 0, VTIME = 1 (0.1s inter-byte timeout)
    termios.c_cc[libc::VMIN] = 0;
    termios.c_cc[libc::VTIME] = 1;

    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) } != 0 {
        return Err(TransportError::PortError("tcsetattr failed".into()));
    }

    Ok(())
}
