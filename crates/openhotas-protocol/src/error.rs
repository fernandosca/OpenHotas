use serde::{Deserialize, Serialize};

/// Errors returned by the firmware in response to malformed or invalid requests.
/// All protocol errors are reported via `Response::Error(ProtocolError)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolError {
    /// Request enum variant not recognized.
    UnknownCommand,
    /// Payload deserialization failed (corrupt or wrong type).
    InvalidPayload,
    /// Frame length field doesn't match payload size.
    InvalidLength,
    /// CRC16 didn't match computed value.
    CrcMismatch,
    /// Protocol version mismatch (major version differs).
    UnsupportedVersion,
    /// Firmware is busy (e.g., calibration in progress) — retry later.
    Busy,
    /// Flash write or erase operation failed.
    FlashError,
    /// Configuration validation failed (e.g., out-of-range value).
    InvalidConfig,
    /// Calibration error (e.g., invalid min/center/max ordering).
    CalibrationError,
    /// Unspecified internal error.
    InternalError,
}
