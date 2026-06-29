//! Runtime configuration types — lightweight snapshot of protocol config
//! delivered from cdc_task to input_task via CONFIG_SIGNAL.
//!
//! No `f32` in protocol (D-06), but the firmware uses f32 internally.
//! Conversion happens here: scaled ints → f32 for filters.
//!
//! D-07: cdc_task signals new config via Signal. input_task checks
//! without blocking.

use crate::axis::AxisConfig;
use crate::calibration::data::CalibrationData;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

/// Symmetric travel limit in firmware-native units.
#[derive(Debug, Clone, Copy)]
pub struct AxisTravelLimitsRuntime {
    pub travel_limit_permille: u16,
}

impl Default for AxisTravelLimitsRuntime {
    fn default() -> Self {
        Self {
            travel_limit_permille: 1000,
        }
    }
}

/// Axis-to-button runtime config (f32 threshold).
#[derive(Debug, Clone, Copy)]
pub struct AxisToButtonRuntime {
    pub enabled: bool,
    pub threshold: f32,
    pub direction: openhotas_protocol::config::AxisDirection,
    pub button_index: u8,
}

impl Default for AxisToButtonRuntime {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.8,
            direction: openhotas_protocol::config::AxisDirection::Both,
            button_index: 0,
        }
    }
}

/// Per-axis runtime config snapshot — everything the input_task needs.
#[derive(Debug, Clone, Copy)]
pub struct AxisRuntimeConfig {
    pub enabled: bool,
    pub inverted: bool,
    pub calibration: CalibrationData,
    pub travel: AxisTravelLimitsRuntime,
    pub filters: AxisConfig,
    pub reset_ema_on_dz: bool,
    pub axis_to_button: AxisToButtonRuntime,
    pub center_offset: f32,
}

impl Default for AxisRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inverted: false,
            calibration: CalibrationData::default(),
            travel: AxisTravelLimitsRuntime::default(),
            filters: AxisConfig::default(),
            reset_ema_on_dz: false,
            axis_to_button: AxisToButtonRuntime::default(),
            center_offset: 0.0,
        }
    }
}

/// Button runtime config.
#[derive(Debug, Clone, Copy)]
pub struct ButtonRuntimeConfig {
    pub enabled_mask: u32,
    pub inverted_mask: u32,
    pub debounce_ms: u8,
}

impl Default for ButtonRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled_mask: 0xFFFF_FFFF,
            inverted_mask: 0xFFFF_FFFF,
            debounce_ms: 5,
        }
    }
}

/// Full runtime configuration — delivered from cdc_task to input_task.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub axes: [AxisRuntimeConfig; 3],
    pub buttons: ButtonRuntimeConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let mut axes = [AxisRuntimeConfig::default(); 3];
        axes[2].reset_ema_on_dz = true; // Twist
        Self {
            axes,
            buttons: ButtonRuntimeConfig::default(),
        }
    }
}

/// Channel: cdc_task sends, input_task receives without blocking.
///
/// D-07: Channel<Capacity=1> + try_recv(), not Mutex.
/// input_task never blocks waiting for config. Old values are discarded
/// if input_task hasn't consumed them yet (capacity 1).
pub static CONFIG_SIGNAL: Channel<CriticalSectionRawMutex, RuntimeConfig, 1> = Channel::new();

/// V1.23 (F1): latest-wins channel send.
///
/// If the channel is full (input_task hasn't consumed the previous config),
/// drain the old pending config and retry with the newest value.
/// Returns `true` if the config was successfully enqueued.
pub fn signal_latest_config(config: RuntimeConfig) -> bool {
    match CONFIG_SIGNAL.try_send(config) {
        Ok(()) => true,
        Err(embassy_sync::channel::TrySendError::Full(cfg)) => {
            // Discard old pending config — latest wins
            let _ = CONFIG_SIGNAL.try_receive();
            CONFIG_SIGNAL.try_send(cfg).is_ok()
        }
    }
}

fn valid_debounce_ms(ms: u8) -> bool {
    matches!(ms, 1 | 2 | 5 | 10 | 20)
}

/// Convert protocol DeviceConfig → firmware RuntimeConfig.
///
/// Scaled integers from protocol → f32 for firmware internals.
/// Returns `None` if validation fails (invalid ranges, version mismatch).
pub fn from_protocol_config(
    cfg: &openhotas_protocol::config::DeviceConfig,
) -> Option<RuntimeConfig> {
    use openhotas_protocol::version::PROTOCOL_VERSION_MAJOR;

    if cfg.protocol_version_major != PROTOCOL_VERSION_MAJOR {
        return None;
    }

    let mut axes = [AxisRuntimeConfig::default(); 3];

    for (i, pa) in cfg.axes.iter().enumerate() {
        // Validate ranges
        if pa.deadzone_permille > 200
            || pa.ema_permille < 1
            || pa.ema_permille > 1000
            || pa.max_jump_raw < 1
            || pa.travel.travel_limit_pct < 1
            || pa.travel.travel_limit_pct > 100
            || !valid_debounce_ms(cfg.buttons.debounce_ms)
            || !(crate::calibration::data::CalibrationData {
                min: pa.calibration.min_raw,
                center: pa.calibration.center_raw,
                max: pa.calibration.max_raw,
            })
            .is_valid(1000)
            // Response curve validation
            || pa.response_curve.point_left.x >= 0
            || pa.response_curve.point_left.x <= -1000
            || pa.response_curve.point_right.x <= 0
            || pa.response_curve.point_right.x >= 1000
            || pa.response_curve.point_left.y < -1000
            || pa.response_curve.point_left.y > 1000
            || pa.response_curve.point_right.y < -1000
            || pa.response_curve.point_right.y > 1000
            // Axis-to-button validation
            || (pa.axis_to_button.enabled && pa.axis_to_button.button_index > 31)
            || (pa.axis_to_button.enabled && pa.axis_to_button.threshold_permille > 1000)
            // Center offset validation
            || pa.center_offset_permille < -200
            || pa.center_offset_permille > 200
        {
            defmt::warn!("Config validation failed for axis {}", i);
            return None;
        }

        axes[i] = AxisRuntimeConfig {
            enabled: pa.enabled,
            inverted: pa.inverted,
            calibration: crate::calibration::data::CalibrationData {
                min: pa.calibration.min_raw,
                center: pa.calibration.center_raw,
                max: pa.calibration.max_raw,
            },
            travel: AxisTravelLimitsRuntime {
                travel_limit_permille: pa.travel.travel_limit_pct as u16 * 10,
            },
            filters: crate::axis::AxisConfig {
                ema_alpha: pa.ema_permille as f32 / 1000.0,
                deadzone: pa.deadzone_permille as f32 / 1000.0,
                // V1.24: scale by calibration range, not full 15-bit (audit #2)
                max_jump: {
                    let cal_range = crate::calibration::data::CalibrationData {
                        min: pa.calibration.min_raw,
                        center: pa.calibration.center_raw,
                        max: pa.calibration.max_raw,
                    }
                    .span()
                    .max(1);
                    pa.max_jump_raw as f32 / cal_range as f32
                },
                response_p1: (
                    pa.response_curve.point_left.x as f32 / 1000.0,
                    pa.response_curve.point_left.y as f32 / 1000.0,
                ),
                response_p3: (
                    pa.response_curve.point_right.x as f32 / 1000.0,
                    pa.response_curve.point_right.y as f32 / 1000.0,
                ),
                // inverted/reset_ema_on_dz carried by AxisRuntimeConfig, not AxisConfig
                ..Default::default()
            },
            reset_ema_on_dz: pa.reset_ema_on_dz,
            axis_to_button: AxisToButtonRuntime {
                enabled: pa.axis_to_button.enabled,
                threshold: pa.axis_to_button.threshold_permille as f32 / 1000.0,
                direction: pa.axis_to_button.direction,
                button_index: pa.axis_to_button.button_index.min(31),
            },
            center_offset: pa.center_offset_permille as f32 / 1000.0,
        };
    }

    // Collision detection: multiple virtual axes must not drive the same button.
    // Physical buttons may share the same bit: the input task composes
    // physical and virtual sources with OR semantics after applying
    // the physical enabled_mask.
    let mut virtual_button_mask: u32 = 0;
    for (i, ax) in axes.iter().enumerate() {
        if ax.axis_to_button.enabled {
            let bit = 1u32 << ax.axis_to_button.button_index;
            if virtual_button_mask & bit != 0 {
                defmt::warn!(
                    "Axis-to-button collision: axis {} and another axis both use button {}",
                    i,
                    ax.axis_to_button.button_index
                );
                return None;
            }
            virtual_button_mask |= bit;
        }
    }

    Some(RuntimeConfig {
        axes,
        buttons: ButtonRuntimeConfig {
            enabled_mask: cfg.buttons.enabled_mask,
            inverted_mask: cfg.buttons.inverted_mask,
            debounce_ms: cfg.buttons.debounce_ms,
        },
    })
}
