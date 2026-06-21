use crate::config::DeviceConfig;
use serde::{Deserialize, Serialize};

/// Axis identifier for calibration and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisId {
    X = 0,
    Y = 1,
    Twist = 2,
}

impl AxisId {
    /// Convert from usize index (0=X, 1=Y, 2=Twist).
    pub fn from_usize(i: usize) -> Self {
        match i {
            0 => Self::X,
            1 => Self::Y,
            _ => Self::Twist,
        }
    }

    /// Parse from string: "x", "y", or "twist" (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        let lower: heapless::String<8> = s.chars().map(|c| c.to_ascii_lowercase()).collect();
        match lower.as_str() {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "twist" => Some(Self::Twist),
            _ => None,
        }
    }
}

/// Calibration point type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationPoint {
    Min,
    Center,
    Max,
}

/// Every request from PC to firmware.
///
/// ## Protocol versioning
/// The first request after CDC connection SHOULD be `GetInfo`.
/// The response contains `DeviceInfo` with protocol version —
/// the PC should check compatibility before sending config commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    // ── System ──────────────────────────────────────────────
    /// Query device identity and protocol version.
    GetInfo,
    /// Trigger software reboot (bootloader stays in application mode).
    Reboot,
    /// Erase stored config and calibration, reboot with defaults.
    FactoryReset,

    // ── Configuration ───────────────────────────────────────
    /// Query current active configuration.
    GetConfig,
    /// Apply new configuration (runtime only, NOT persisted).
    SetConfig(DeviceConfig),
    /// Persist current configuration to flash.
    SaveConfig,
    /// Reload defaults into runtime config (does NOT write flash).
    LoadDefaults,

    // ── Diagnostics ─────────────────────────────────────────
    GetRawAxes,
    GetProcessedAxes,
    GetButtonStates,
    GetSensorStatus,
    GetRuntimeStats,
    GetErrorCounters,

    // ── Calibration ─────────────────────────────────────────
    /// Start calibration session for the given axis.
    StartCalibration(AxisId),
    /// Capture a calibration point for an axis (Min/Center/Max).
    CaptureCalibrationPoint {
        axis: AxisId,
        point: CalibrationPoint,
    },
    /// Finish calibration for the given axis.
    /// The new calibration is applied to runtime config but NOT persisted
    /// (use SaveConfig to persist).
    FinishCalibration(AxisId),
}
