use serde::{Deserialize, Serialize};

/// Full device configuration.
///
/// Sent/received via `Request::SetConfig` / `Response::Config`.
/// The firmware validates `protocol_version_major == PROTOCOL_VERSION_MAJOR`
/// before accepting any config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub protocol_version_major: u8,
    pub protocol_version_minor: u8,
    pub axes: [AxisConfig; 3],
    pub buttons: ButtonConfig,
}

/// Per-axis configuration for filters, calibration, and travel limits.
///
/// ### Scales (D-06 — no f32)
/// - `deadzone_permille`: 0..200  →  0.000..0.200
/// - `ema_permille`: 1..1000      →  0.001..1.000
/// - `max_jump_raw`: 1..32767     →  raw sensor delta threshold
/// - `response_curve`: piecewise linear with 5 control points (permille)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisConfig {
    pub enabled: bool,
    pub inverted: bool,

    pub calibration: CalibrationData,
    pub travel: AxisTravelLimits,

    pub deadzone_permille: u16,
    pub ema_permille: u16,
    pub max_jump_raw: u16,

    /// Piecewise linear response curve. Replaces expo filter.
    pub response_curve: ResponseCurveData,

    /// Reset EMA filter when entering deadzone (used for Twist axis).
    pub reset_ema_on_dz: bool,

    /// Axis-to-button mapping for this axis.
    pub axis_to_button: AxisToButtonConfig,

    /// Center offset in permille (-200..200). Adjusts the zero point.
    /// Positive values shift center to the right, negative to the left.
    /// Applied after calibration, before travel limits.
    pub center_offset_permille: i16,
}

/// Control point for the piecewise linear response curve.
///
/// Values are in permille: -1000..1000 maps to -1.0..1.0.
/// x: position on the input axis, y: output value at that position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: i16,
    pub y: i16,
}

/// Piecewise linear response curve with 5 control points.
///
/// P0=(-1000,-1000), P2=(0,0), P4=(1000,1000) are fixed (endpoints + center).
/// `point_left` (P1) and `point_right` (P3) are the variable control points.
///
/// Constraints:
/// - `point_left.x` must be in (-1000, 0) — between left endpoint and center
/// - `point_right.x` must be in (0, 1000) — between center and right endpoint
/// - `y` values in [-1000, 1000] for both points
///
/// Default: P1=(-500,-500), P3=(500,500) → straight diagonal (linear response).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseCurveData {
    pub point_left: CurvePoint,
    pub point_right: CurvePoint,
}

/// Raw calibration limits for a single axis (u16, 15-bit sensor).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CalibrationData {
    pub min_raw: u16,
    pub center_raw: u16,
    pub max_raw: u16,
}

/// Symmetric physical travel limit as percentage of each side from center.
///
/// After calibration, the firmware treats the center as 0.0 and applies the
/// same limit to both directions. Example:
/// - `travel_limit_pct = 95` → ±95% of calibrated travel maps to full output
/// - `travel_limit_pct = 100` → full calibrated travel
///
/// This solves the problem of "physically reached endstop but PC shows < 100%".
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisTravelLimits {
    pub travel_limit_pct: u8,
}

/// Button configuration for 32 buttons (2× MCP23S17).
///
/// With pull-up resistors, unpressed = 1, pressed = 0.
/// Default recommendation: enable all, invert all.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ButtonConfig {
    /// Bits set to 1 = button enabled. Default: 0xFFFFFFFF (all enabled).
    pub enabled_mask: u32,
    /// Bits set to 1 = inverted logic. Default: 0xFFFFFFFF (all inverted).
    pub inverted_mask: u32,
    /// Debounce time in milliseconds. Allowed values: 1, 2, 5, 10, 20.
    pub debounce_ms: u8,
}

/// Direction for axis-to-button mapping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AxisDirection {
    /// Activate when axis > threshold (positive direction)
    Positive,
    /// Activate when axis < -threshold (negative direction)
    Negative,
    /// Activate when |axis| > threshold (both directions)
    Both,
}

/// Maps an axis position to a virtual button press.
///
/// When the axis value exceeds the threshold in the specified direction,
/// the corresponding bit is set in the button mask. The axis itself
/// continues to function normally.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisToButtonConfig {
    /// Enable this mapping. Default: false.
    pub enabled: bool,
    /// Threshold in permille (0..1000). Maps to 0.0..1.0 in firmware.
    pub threshold_permille: u16,
    /// Direction that activates the button.
    pub direction: AxisDirection,
    /// Button index (0..31) to set when threshold is exceeded.
    pub button_index: u8,
}

impl Default for AxisToButtonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_permille: 800,
            direction: AxisDirection::Both,
            button_index: 0,
        }
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        let mut axes = [AxisConfig::default(); 3];
        // Twist axis (index 2) resets EMA on deadzone entry by default
        axes[2].reset_ema_on_dz = true;
        Self {
            protocol_version_major: crate::version::PROTOCOL_VERSION_MAJOR,
            protocol_version_minor: crate::version::PROTOCOL_VERSION_MINOR,
            axes,
            buttons: ButtonConfig::default(),
        }
    }
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inverted: false,
            calibration: CalibrationData::default(),
            travel: AxisTravelLimits::default(),
            deadzone_permille: 20, // 2.0%
            ema_permille: 300,     // alpha = 0.3
            max_jump_raw: 4915,    // ~0.15 of 32767
            response_curve: ResponseCurveData::default(),
            reset_ema_on_dz: false,
            axis_to_button: AxisToButtonConfig::default(),
            center_offset_permille: 0,
        }
    }
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            min_raw: 0,
            center_raw: 16384,
            max_raw: 32767,
        }
    }
}

impl Default for AxisTravelLimits {
    fn default() -> Self {
        Self {
            travel_limit_pct: 100,
        }
    }
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            enabled_mask: 0xFFFF_FFFF,
            inverted_mask: 0xFFFF_FFFF,
            debounce_ms: 5,
        }
    }
}

impl Default for ResponseCurveData {
    fn default() -> Self {
        Self {
            point_left: CurvePoint { x: -500, y: -500 },
            point_right: CurvePoint { x: 500, y: 500 },
        }
    }
}

/// Response curve presets — centralizados para firmware, CLI e GUI.
pub mod presets {
    use super::{CurvePoint, ResponseCurveData};

    pub const LINEAR: ResponseCurveData = ResponseCurveData {
        point_left: CurvePoint { x: -500, y: -500 },
        point_right: CurvePoint { x: 500, y: 500 },
    };

    pub const SMOOTH: ResponseCurveData = ResponseCurveData {
        point_left: CurvePoint { x: -400, y: -250 },
        point_right: CurvePoint { x: 400, y: 250 },
    };

    pub const CENTER: ResponseCurveData = ResponseCurveData {
        point_left: CurvePoint { x: -300, y: -150 },
        point_right: CurvePoint { x: 300, y: 150 },
    };

    pub const S_CURVE: ResponseCurveData = ResponseCurveData {
        point_left: CurvePoint { x: -250, y: -600 },
        point_right: CurvePoint { x: 250, y: 600 },
    };
}

/// Curve preset enumeration — type-safe alternative to string matching.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurvePreset {
    Linear,
    Smooth,
    Center,
    SCurve,
}

impl CurvePreset {
    /// Convert preset to ResponseCurveData.
    pub fn to_response_curve(self) -> ResponseCurveData {
        match self {
            Self::Linear => presets::LINEAR,
            Self::Smooth => presets::SMOOTH,
            Self::Center => presets::CENTER,
            Self::SCurve => presets::S_CURVE,
        }
    }

    /// Parse preset from name string (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "linear" | "Linear" | "LINEAR" => Some(Self::Linear),
            "smooth" | "Smooth" | "SMOOTH" => Some(Self::Smooth),
            "center" | "Center" | "CENTER" => Some(Self::Center),
            "s" | "S" | "s-curve" | "S-curve" | "S-Curve" | "S-CURVE" => Some(Self::SCurve),
            _ => None,
        }
    }

    /// Get all available preset names.
    pub fn all_names() -> &'static [&'static str] {
        &["linear", "smooth", "center", "s"]
    }
}
