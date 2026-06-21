//! CLI command implementations — one function per CLI command.
//!
//! Each function takes an `OpenHotasTransport` and optional parameters,
//! sends the appropriate request(s), and returns a formatted string
//! for display.

use crate::display;
use crate::transport::{OpenHotasTransport, TransportError};
use openhotas_protocol::diagnostics::SensorStatusReport;
use openhotas_protocol::error::ProtocolError;
use openhotas_protocol::request::{AxisId, CalibrationPoint, Request};
use openhotas_protocol::response::Response;
use std::io::{self, BufRead, Write};

pub struct SetAxisOptions {
    pub deadzone_pct: Option<u8>,
    pub invert: Option<bool>,
    pub enabled: Option<bool>,
    pub ema_pct: Option<u8>,
    pub travel_limit_pct: Option<u8>,
    pub curve_preset: Option<String>,
    pub center_offset: Option<i16>,
    pub axis_to_button: Option<String>,
}

fn require_pct_u8(name: &str, value: u8, min: u8, max: u8) -> Result<u8, TransportError> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(format!("{name} must be in {min}..={max}, got {value}").into())
    }
}

fn axis_is_healthy(status: &SensorStatusReport, axis: AxisId) -> bool {
    match axis {
        AxisId::X => status.x.healthy,
        AxisId::Y => status.y.healthy,
        AxisId::Twist => status.twist.healthy,
    }
}

// ── Read-only Commands ───────────────────────────────────────────────────

pub fn cmd_info(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetInfo)? {
        Response::Info(info) => Ok(display::format_info(&info)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_get_config(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetConfig)? {
        Response::Config(cfg) => Ok(display::format_config(&cfg)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_raw_axes(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetRawAxes)? {
        Response::RawAxes(raw) => Ok(display::format_raw_axes(&raw)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_processed_axes(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetProcessedAxes)? {
        Response::ProcessedAxes(p) => Ok(display::format_processed_axes(&p)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_buttons(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetButtonStates)? {
        Response::ButtonStates(b) => Ok(display::format_buttons(&b)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_stats(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetRuntimeStats)? {
        Response::RuntimeStats(s) => Ok(display::format_stats(&s)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_sensor_status(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetSensorStatus)? {
        Response::SensorStatus(s) => Ok(display::format_sensor_status(&s)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_errors(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::GetErrorCounters)? {
        Response::ErrorCounters(e) => Ok(display::format_errors(&e)),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

// ── Config Write Commands ────────────────────────────────────────────────

use openhotas_protocol::config::{AxisDirection, CurvePreset};

/// Curve preset definitions — delegates to protocol crate.
fn curve_preset(name: &str) -> Result<CurvePreset, TransportError> {
    CurvePreset::from_name(name).ok_or_else(|| {
        format!(
            "Unknown curve preset: {name}. Use {}.",
            CurvePreset::all_names().join(", ")
        )
        .into()
    })
}

/// Parse axis-to-button config from string format: "enabled=true,threshold=80,direction=positive,button=28"
fn parse_axis_to_button(
    s: &str,
) -> Result<openhotas_protocol::config::AxisToButtonConfig, TransportError> {
    let mut enabled = false;
    let mut threshold = 800u16;
    let mut direction = AxisDirection::Both;
    let mut button = 0u8;

    for part in s.split(',') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("").trim();
        let val = kv.next().unwrap_or("").trim();

        match key {
            "enabled" => {
                enabled = val.parse::<bool>().map_err(|_| {
                    TransportError::PortError(format!("Invalid enabled value: {val}"))
                })?;
            }
            "threshold" => {
                threshold = val.parse::<u16>().map_err(|_| {
                    TransportError::PortError(format!("Invalid threshold value: {val}"))
                })?;
                if threshold > 1000 {
                    return Err(TransportError::PortError(
                        "threshold must be 0..1000".to_string(),
                    ));
                }
            }
            "direction" => {
                direction = match val.to_lowercase().as_str() {
                    "positive" => AxisDirection::Positive,
                    "negative" => AxisDirection::Negative,
                    "both" => AxisDirection::Both,
                    _ => {
                        return Err(TransportError::PortError(format!(
                            "Invalid direction: {val}. Use positive, negative, or both"
                        )));
                    }
                };
            }
            "button" => {
                button = val.parse::<u8>().map_err(|_| {
                    TransportError::PortError(format!("Invalid button value: {val}"))
                })?;
                if button > 31 {
                    return Err(TransportError::PortError(
                        "button must be 0..31".to_string(),
                    ));
                }
            }
            _ => {
                return Err(TransportError::PortError(format!("Unknown key: {key}")));
            }
        }
    }

    Ok(openhotas_protocol::config::AxisToButtonConfig {
        enabled,
        threshold_permille: threshold,
        direction,
        button_index: button,
    })
}

/// Set an axis property: read current config, modify one field, send back.
pub fn cmd_set_axis(
    t: &mut OpenHotasTransport,
    axis: AxisId,
    options: SetAxisOptions,
) -> Result<String, TransportError> {
    // Get current config
    let mut config = match t.send(Request::GetConfig)? {
        Response::Config(cfg) => cfg,
        other => return Ok(format!("Unexpected response: {other:?}")),
    };

    let ax = &mut config.axes[axis as usize];

    if let Some(dz) = options.deadzone_pct {
        let dz = require_pct_u8("--deadzone", dz, 0, 20)?;
        ax.deadzone_permille = (dz as u16).saturating_mul(10); // pct → permille (0-200)
    }
    if let Some(inv) = options.invert {
        ax.inverted = inv;
    }
    if let Some(en) = options.enabled {
        ax.enabled = en;
    }
    if let Some(ema) = options.ema_pct {
        let ema = require_pct_u8("--ema", ema, 1, 100)?;
        ax.ema_permille = (ema as u16).saturating_mul(10); // pct → permille
    }
    if let Some(limit) = options.travel_limit_pct {
        ax.travel.travel_limit_pct = require_pct_u8("--travel-limit", limit, 1, 100)?;
    }
    if let Some(preset_name) = options.curve_preset {
        let preset = curve_preset(&preset_name)?;
        ax.response_curve = preset.to_response_curve();
    }
    if let Some(offset) = options.center_offset {
        if !(-200..=200).contains(&offset) {
            return Err("--center-offset must be in -200..200".to_string().into());
        }
        ax.center_offset_permille = offset;
    }
    if let Some(atb_str) = options.axis_to_button {
        let atb = parse_axis_to_button(&atb_str)?;
        ax.axis_to_button = atb;
    }

    // Send new config
    match t.send(Request::SetConfig(config.clone()))? {
        Response::Ack => {
            // Confirm by reading back
            match t.send(Request::GetConfig)? {
                Response::Config(confirmed) => Ok(display::format_axis_config(
                    axis,
                    &confirmed.axes[axis as usize],
                )),
                other => Ok(format!("Config applied, but read-back failed: {other:?}")),
            }
        }
        Response::Error(e) => Ok(format!("Device rejected config: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_save(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::SaveConfig)? {
        Response::Ack => Ok("Configuration saved to flash.".into()),
        Response::Error(e) => Ok(format!("Save failed: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_load_defaults(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::LoadDefaults)? {
        Response::Ack => {
            Ok("Defaults loaded (runtime only, not persisted). Use 'save' to persist.".into())
        }
        Response::Error(e) => Ok(format!("LoadDefaults failed: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_reboot(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::Reboot)? {
        Response::Ack => {
            Ok("Reboot acknowledged. Device will disconnect and reconnect in ~100ms.".into())
        }
        Response::Error(e) => Ok(format!("Reboot failed: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

pub fn cmd_factory_reset(t: &mut OpenHotasTransport) -> Result<String, TransportError> {
    match t.send(Request::FactoryReset)? {
        Response::Ack => Ok(
            "Factory reset acknowledged. Device will reboot with defaults. Reconnect after ~200ms."
                .into(),
        ),
        Response::Error(e) => Ok(format!("Factory reset failed: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}

// ── Calibration ──────────────────────────────────────────────────────────

/// Interactive calibration wizard for a single axis.
pub fn cmd_calibrate(t: &mut OpenHotasTransport, axis: AxisId) -> Result<String, TransportError> {
    let axis_name = display::axis_name(axis);

    match t.send(Request::GetSensorStatus)? {
        Response::SensorStatus(status) if axis_is_healthy(&status, axis) => {}
        Response::SensorStatus(_) => {
            return Ok(format!(
                "{axis_name} is unhealthy. Fix sensor status before calibration."
            ));
        }
        other => return Ok(format!("Unexpected sensor-status response: {other:?}")),
    }

    // Start calibration session
    match t.send(Request::StartCalibration(axis))? {
        Response::Ack => {}
        Response::Error(ProtocolError::Busy) => {
            return Ok("Calibration already in progress for another axis. Finish it first.".into());
        }
        Response::Error(e) => return Ok(format!("StartCalibration failed: {e:?}")),
        other => return Ok(format!("Unexpected response: {other:?}")),
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    // Point: Min
    println!("\n  Move {axis_name} to MINIMUM position and press Enter...");
    stdout.flush().ok();
    line.clear();
    stdin.lock().read_line(&mut line).ok();
    match t.send(Request::CaptureCalibrationPoint {
        axis,
        point: CalibrationPoint::Min,
    })? {
        Response::Ack => println!("  ✓ Min captured"),
        Response::Error(e) => {
            let _ = t.send(Request::FinishCalibration(axis));
            return Ok(format!("Capture failed: {e:?}"));
        }
        other => return Ok(format!("Unexpected response: {other:?}")),
    }

    // Point: Center
    println!("  Center {axis_name} and press Enter...");
    stdout.flush().ok();
    line.clear();
    stdin.lock().read_line(&mut line).ok();
    match t.send(Request::CaptureCalibrationPoint {
        axis,
        point: CalibrationPoint::Center,
    })? {
        Response::Ack => println!("  ✓ Center captured"),
        Response::Error(e) => {
            let _ = t.send(Request::FinishCalibration(axis));
            return Ok(format!("Capture failed: {e:?}"));
        }
        other => return Ok(format!("Unexpected response: {other:?}")),
    }

    // Point: Max
    println!("  Move {axis_name} to MAXIMUM position and press Enter...");
    stdout.flush().ok();
    line.clear();
    stdin.lock().read_line(&mut line).ok();
    match t.send(Request::CaptureCalibrationPoint {
        axis,
        point: CalibrationPoint::Max,
    })? {
        Response::Ack => println!("  ✓ Max captured"),
        Response::Error(e) => {
            let _ = t.send(Request::FinishCalibration(axis));
            return Ok(format!("Capture failed: {e:?}"));
        }
        other => return Ok(format!("Unexpected response: {other:?}")),
    }

    // Finish calibration
    match t.send(Request::FinishCalibration(axis))? {
        Response::Ack => {
            // Read back the new calibration
            if let Response::Config(cfg) = t.send(Request::GetConfig)? {
                let cal = &cfg.axes[axis as usize].calibration;
                Ok(format!(
                    "✓ Calibration complete for {axis_name}\n  min={} center={} max={}\n  Run 'save' to persist.",
                    cal.min_raw, cal.center_raw, cal.max_raw,
                ))
            } else {
                Ok("✓ Calibration complete. Run 'save' to persist.".into())
            }
        }
        Response::Error(e) => Ok(format!("FinishCalibration failed: {e:?}")),
        other => Ok(format!("Unexpected response: {other:?}")),
    }
}
