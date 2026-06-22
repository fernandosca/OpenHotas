use serde::{Deserialize, Serialize};

/// Diagnostic snapshot — read by `Request::GetRuntimeStats`.
///
/// All values are unsigned 32-bit counters. The firmware resets them
/// on reboot; the PC should interpret them as "since boot".
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub reports_sent: u32,
    pub send_errors: u32,
    pub sensor_cycles: u32,
    pub last_cycle_us: u32,
    pub max_cycle_us: u32,
}

/// Raw sensor values for all 3 axes. Read by `Request::GetRawAxes`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RawAxes {
    pub x: u16,
    pub y: u16,
    pub twist: u16,
}

/// Processed (post-pipeline) axis values, scaled to i16 HID range.
/// Read by `Request::GetProcessedAxes`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProcessedAxes {
    /// Value in [-32767, 32767] — same range as HID report.
    pub x: i16,
    pub y: i16,
    pub twist: i16,
    /// Sensor health flags (bit 0=X, bit 1=Y, bit 2=Twist; 0=healthy).
    pub unhealthy_mask: u8,
}

/// Current button state (32 buttons packed as u32 bitmask).
/// Read by `Request::GetButtonStates`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ButtonStates {
    /// Bit N = 1 means button N is pressed (after config inversion applied).
    pub mask: u32,
}

/// Per-sensor status report. Read by `Request::GetSensorStatus`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SensorStatusReport {
    pub x: SensorInfo,
    pub y: SensorInfo,
    pub twist: SensorInfo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SensorInfo {
    pub healthy: bool,
    pub error_count: u32,
}

/// Error counters snapshot. Read by `Request::GetErrorCounters`.
///
/// V1.23: separated protocol CRC (CDC frame) from sensor CRC (MT6826S encoder).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ErrorCounters {
    /// CRC errors on the CDC binary protocol frame.
    pub protocol_crc_errors: u32,
    /// CRC errors from the MT6826S sensor (per-encoder SPI CRC8).
    pub sensor_crc_errors: u32,
    /// Weak magnet / under-voltage warnings from any sensor.
    pub magnet_errors: u32,
    /// Flash erase/write/CRC failures.
    pub flash_errors: u32,
    /// MCP23S17 init/read failures.
    pub button_errors: u32,
    /// True when button expander is unavailable and firmware reports released buttons.
    pub buttons_degraded: bool,
}
