use crate::config::DeviceConfig;
use crate::diagnostics::{
    ButtonStates, ErrorCounters, ProcessedAxes, RawAxes, RuntimeStats, SensorStatusReport,
};
use crate::error::ProtocolError;
use serde::{Deserialize, Serialize};

/// Device identity, returned in response to `Request::GetInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Firmware SemVer string, e.g. "1.2.2".
    pub firmware_version: [u8; 8],
    /// Git hash (7-char short), zero-padded.
    pub git_hash: [u8; 8],
    /// Protocol version.
    pub protocol_major: u8,
    pub protocol_minor: u8,
    /// Hardware: always 3 axes, 32 buttons.
    pub axis_count: u8,
    pub button_count: u8,
}

/// Every response from firmware to PC.
///
/// The firmware always responds to a `Request` with exactly one `Response`.
/// Timeout and retry logic is the PC's responsibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Command succeeded with no data payload.
    Ack,
    /// Command failed — see `ProtocolError` variant.
    Error(ProtocolError),

    // ── System ──
    Info(DeviceInfo),

    // ── Configuration ──
    Config(DeviceConfig),

    // ── Diagnostics ──
    RawAxes(RawAxes),
    ProcessedAxes(ProcessedAxes),
    ButtonStates(ButtonStates),
    SensorStatus(SensorStatusReport),
    RuntimeStats(RuntimeStats),
    ErrorCounters(ErrorCounters),
}
