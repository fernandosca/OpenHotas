//! CDC protocol request handlers — dispatched by cdc_task.
//!
//! Each handler group has a single responsibility:
//! - handle_read_request: diagnostics & config (read-only)
//! - handle_write_request: set/save/load config (mutates state)
//! - handle_calibration_request: calibration session management
//!
//! Shared state (active_config, cal_session, pending_reboot) is
//! owned by cdc_task and passed via &mut references.

use core::sync::atomic::Ordering;

use openhotas_protocol::config::DeviceConfig;
use openhotas_protocol::diagnostics::{
    ButtonStates, ErrorCounters, ProcessedAxes, RawAxes, RuntimeStats, SensorInfo,
    SensorStatusReport,
};
use openhotas_protocol::error::ProtocolError;
use openhotas_protocol::request::{AxisId, CalibrationPoint, Request};
use openhotas_protocol::response::{DeviceInfo, Response};
use openhotas_protocol::version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};

use crate::config::runtime::{self, signal_latest_config};
use crate::config::stored_config_v2;
use crate::constants::{FIRMWARE_GIT_HASH, FIRMWARE_VERSION, MT6826_ANGLE_MAX};
use crate::diagnostics::runtime_stats;

use crate::tasks::cdc::{CalibrationSession, PendingReset};

// ── Helpers ──────────────────────────────────────────────────────────────

/// Pack a firmware version string into a fixed-size [u8; 8] field.
pub fn version_to_bytes(s: &str) -> [u8; 8] {
    let mut buf = [0u8; 8];
    let bytes = s.as_bytes();
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

// ── Read-Only Handler ────────────────────────────────────────────────────

pub fn handle_read_request(req: &Request, active_config: &DeviceConfig) -> Response {
    match req {
        Request::GetInfo => {
            let info = DeviceInfo {
                firmware_version: version_to_bytes(FIRMWARE_VERSION),
                git_hash: version_to_bytes(FIRMWARE_GIT_HASH),
                protocol_major: PROTOCOL_VERSION_MAJOR,
                protocol_minor: PROTOCOL_VERSION_MINOR,
                axis_count: 3,
                button_count: 32,
            };
            Response::Info(info)
        }
        Request::GetConfig => Response::Config(active_config.clone()),
        Request::GetRawAxes => {
            let raw = RawAxes {
                x: runtime_stats::RAW_AXIS_X.load(Ordering::Relaxed) as u16,
                y: runtime_stats::RAW_AXIS_Y.load(Ordering::Relaxed) as u16,
                twist: runtime_stats::RAW_AXIS_TWIST.load(Ordering::Relaxed) as u16,
            };
            Response::RawAxes(raw)
        }
        Request::GetProcessedAxes => {
            let proc = ProcessedAxes {
                // V1.25: AtomicI32 — direct i32→i16 cast preserves sign correctly
                x: runtime_stats::PROC_AXIS_X.load(Ordering::Relaxed) as i16,
                y: runtime_stats::PROC_AXIS_Y.load(Ordering::Relaxed) as i16,
                twist: runtime_stats::PROC_AXIS_TWIST.load(Ordering::Relaxed) as i16,
                unhealthy_mask: runtime_stats::SENSOR_UNHEALTHY.load(Ordering::Relaxed),
            };
            Response::ProcessedAxes(proc)
        }
        Request::GetButtonStates => Response::ButtonStates(ButtonStates {
            mask: runtime_stats::BUTTON_MASK.load(Ordering::Relaxed),
        }),
        Request::GetSensorStatus => {
            let unhealthy = runtime_stats::SENSOR_UNHEALTHY.load(Ordering::Relaxed);
            Response::SensorStatus(SensorStatusReport {
                x: SensorInfo {
                    healthy: unhealthy & 0x01 == 0,
                    error_count: runtime_stats::SENSOR_X_ERRORS.load(Ordering::Relaxed),
                },
                y: SensorInfo {
                    healthy: unhealthy & 0x02 == 0,
                    error_count: runtime_stats::SENSOR_Y_ERRORS.load(Ordering::Relaxed),
                },
                twist: SensorInfo {
                    healthy: unhealthy & 0x04 == 0,
                    error_count: runtime_stats::SENSOR_TWIST_ERRORS.load(Ordering::Relaxed),
                },
            })
        }
        Request::GetRuntimeStats => Response::RuntimeStats(RuntimeStats {
            reports_sent: runtime_stats::REPORTS_SENT.load(Ordering::Relaxed),
            send_errors: runtime_stats::SEND_ERRORS.load(Ordering::Relaxed),
            sensor_cycles: runtime_stats::SENSOR_CYCLES.load(Ordering::Relaxed),
            last_cycle_us: runtime_stats::LAST_CYCLE_US.load(Ordering::Relaxed),
            max_cycle_us: runtime_stats::MAX_CYCLE_US.load(Ordering::Relaxed),
        }),
        Request::GetErrorCounters => Response::ErrorCounters(ErrorCounters {
            protocol_crc_errors: runtime_stats::PROTOCOL_CRC_ERRORS.load(Ordering::Relaxed),
            sensor_crc_errors: runtime_stats::SENSOR_CRC_ERRORS.load(Ordering::Relaxed),
            magnet_errors: runtime_stats::MAGNET_ERRORS.load(Ordering::Relaxed),
            flash_errors: runtime_stats::FLASH_ERRORS.load(Ordering::Relaxed),
            button_errors: runtime_stats::BUTTON_ERRORS.load(Ordering::Relaxed),
            buttons_degraded: runtime_stats::BUTTONS_DEGRADED.load(Ordering::Relaxed) != 0,
        }),
        _ => Response::Error(ProtocolError::InvalidPayload),
    }
}

// ── Write Handler ────────────────────────────────────────────────────────

pub fn handle_write_request(
    req: &Request,
    active_config: &mut DeviceConfig,
    pending_reset: &mut PendingReset,
) -> Response {
    match req {
        Request::SetConfig(cfg) => match runtime::from_protocol_config(cfg) {
            Some(runtime_cfg) => {
                if signal_latest_config(runtime_cfg) {
                    *active_config = cfg.clone();
                    Response::Ack
                } else {
                    Response::Error(ProtocolError::InternalError)
                }
            }
            None => Response::Error(ProtocolError::InvalidConfig),
        },
        Request::SaveConfig => match stored_config_v2::save_config(active_config) {
            Ok(()) => Response::Ack,
            Err(_) => Response::Error(ProtocolError::FlashError),
        },
        Request::LoadDefaults => {
            let defaults = DeviceConfig::default();
            if let Some(runtime_cfg) = runtime::from_protocol_config(&defaults) {
                signal_latest_config(runtime_cfg);
            }
            *active_config = defaults;
            Response::Ack
        }
        Request::FactoryReset => {
            let defaults = DeviceConfig::default();
            if stored_config_v2::save_config(&defaults).is_err() {
                return Response::Error(ProtocolError::FlashError);
            }
            if let Some(runtime_cfg) = runtime::from_protocol_config(&defaults) {
                signal_latest_config(runtime_cfg);
            }
            *active_config = defaults;
            *pending_reset = PendingReset::Application;
            Response::Ack
        }
        Request::Reboot => {
            *pending_reset = PendingReset::Application;
            Response::Ack
        }
        Request::RebootToBootloader => {
            *pending_reset = PendingReset::UsbBoot;
            Response::Ack
        }
        _ => Response::Error(ProtocolError::InvalidPayload),
    }
}

// ── Calibration Handler ──────────────────────────────────────────────────

pub fn handle_calibration_request(
    req: &Request,
    active_config: &mut DeviceConfig,
    cal_session: &mut Option<CalibrationSession>,
) -> Response {
    match req {
        Request::StartCalibration(axis) => {
            if cal_session.is_some() {
                return Response::Error(ProtocolError::Busy);
            }
            *cal_session = Some(CalibrationSession {
                axis: *axis,
                min: None,
                center: None,
                max: None,
            });
            Response::Ack
        }
        Request::CaptureCalibrationPoint { axis, point } => {
            let session = match cal_session {
                Some(s) if s.axis == *axis => s,
                _ => return Response::Error(ProtocolError::CalibrationError),
            };
            // Snapshot atômico: lê SENSOR_UNHEALTHY + RAW_AXIS_* no mesmo
            // critical_section para garantir que ambos são do mesmo ciclo do
            // input_task. Sem isso, um cenário de borda poderia ler healthy=true
            // de um ciclo e raw de outro — capturando um valor corrompido.
            // critical_section desativa interrupts, o que é suficiente porque
            // o RP2350 roda single-core (embassy num core, segundo core desligado).
            let (healthy, raw) = critical_section::with(|_| {
                let unhealthy = runtime_stats::SENSOR_UNHEALTHY.load(Ordering::Relaxed);
                let mask = match axis {
                    AxisId::X => 0x01,
                    AxisId::Y => 0x02,
                    AxisId::Twist => 0x04,
                };
                let healthy = unhealthy & mask == 0;
                let raw = match axis {
                    AxisId::X => runtime_stats::RAW_AXIS_X.load(Ordering::Relaxed) as u16,
                    AxisId::Y => runtime_stats::RAW_AXIS_Y.load(Ordering::Relaxed) as u16,
                    AxisId::Twist => runtime_stats::RAW_AXIS_TWIST.load(Ordering::Relaxed) as u16,
                };
                (healthy, raw)
            });
            if !healthy {
                return Response::Error(ProtocolError::CalibrationError);
            }
            if raw > MT6826_ANGLE_MAX {
                return Response::Error(ProtocolError::CalibrationError);
            }
            match point {
                CalibrationPoint::Min => session.min = Some(raw),
                CalibrationPoint::Center => session.center = Some(raw),
                CalibrationPoint::Max => session.max = Some(raw),
            }
            Response::Ack
        }
        Request::FinishCalibration(axis) => {
            let session = match cal_session.take() {
                Some(s) if s.axis == *axis => s,
                _ => return Response::Error(ProtocolError::CalibrationError),
            };
            let min = session.min.ok_or(ProtocolError::CalibrationError);
            let center = session.center.ok_or(ProtocolError::CalibrationError);
            let max = session.max.ok_or(ProtocolError::CalibrationError);
            match (min, center, max) {
                (Ok(min), Ok(center), Ok(max)) => {
                    let calibration =
                        crate::calibration::data::CalibrationData { min, center, max };
                    if !calibration.is_valid(1000) {
                        return Response::Error(ProtocolError::CalibrationError);
                    }
                    let idx = *axis as usize;
                    active_config.axes[idx].calibration.min_raw = min;
                    active_config.axes[idx].calibration.center_raw = center;
                    active_config.axes[idx].calibration.max_raw = max;
                    if let Some(runtime_cfg) = runtime::from_protocol_config(active_config) {
                        signal_latest_config(runtime_cfg);
                    }
                    // NOTA: FinishCalibration NÃO salva em flash. O host (GUI)
                    // deve enviar SaveConfig separadamente para persistir.
                    // Isso evita desgaste desnecessário da flash durante
                    // calibração interativa (múltiplos Start/Finish).
                    Response::Ack
                }
                _ => Response::Error(ProtocolError::CalibrationError),
            }
        }
        _ => Response::Error(ProtocolError::InvalidPayload),
    }
}
