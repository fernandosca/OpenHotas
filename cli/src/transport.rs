//! Transport layer — discovers CDC port, sends requests, receives responses.
//!
//! Cross-platform serial transport (Linux / Windows / macOS).

use openhotas_protocol::frame::{crc16_ccitt, FrameParser, MAX_PAYLOAD_SIZE, SOF_A, SOF_B};
use openhotas_protocol::request::Request;
use openhotas_protocol::response::{DeviceInfo, Response};
use openhotas_protocol::version::PROTOCOL_VERSION_MAJOR;
use std::io::{Read, Write};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

/// Connection to the OpenHOTAS CDC port.
pub struct OpenHotasTransport {
    port: Box<dyn serialport::SerialPort>,
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
            Self::DeviceNotFound => write!(f, "No OpenHOTAS device found. Is it connected?"),
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
        let ports = serialport::available_ports()
            .map_err(|e| TransportError::PortError(format!("cannot list serial ports: {e}")))?;
        let found_port = !ports.is_empty();
        for port in ports {
            if let Some(transport) = Self::try_open_openhotas(&port.port_name) {
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

    fn try_open_openhotas(path: &str) -> Option<Self> {
        let mut transport = Self::open(path).ok()?;
        transport.validate_openhotas().ok()?;
        Some(transport)
    }

    fn open(path: &str) -> Result<Self, TransportError> {
        let port = serialport::new(path, 115_200)
            .timeout(Duration::from_millis(20))
            .open()
            .map_err(|e| TransportError::PortError(format!("Cannot open {path}: {e}")))?;

        Ok(Self {
            port,
            parser: FrameParser::new(),
        })
    }

    /// Send a request and wait for the response.
    pub fn send(&mut self, request: Request) -> Result<Response, TransportError> {
        let frame = self.build_request_frame(&request)?;
        self.port
            .write_all(&frame)
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        self.port
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

            match self.port.read(&mut byte_buf) {
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
                Err(ref e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Non-blocking, no data yet — brief sleep then retry
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => return Err(TransportError::ReadError(e.to_string())),
            }
        }
    }
}
