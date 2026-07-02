//! CDC configuration task — binary protocol handler.
//!
//! D-02: request/response, no spontaneous telemetry.
//! D-08: cdc_task owns the active configuration, not a global Mutex.
//!
//! Architecture:
//! - cdc.rs: frame I/O + task loop + request dispatch
//! - cdc_handlers.rs: request-specific logic (read/write/calibration)

use core::sync::atomic::Ordering;

use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{Receiver, Sender};
use embassy_usb::driver::EndpointError;

use openhotas_protocol::frame::{FrameError, FrameParser, MAX_PAYLOAD_SIZE, SOF_A, SOF_B};
use openhotas_protocol::request::Request;
use openhotas_protocol::response::Response;

use crate::config::runtime::{self, signal_latest_config};
use crate::config::stored_config_v2;
use crate::diagnostics::runtime_stats;
use crate::tasks::cdc_handlers::{
    handle_calibration_request, handle_read_request, handle_write_request,
};

// ── Calibration Session ──────────────────────────────────────────────────

/// Tracks the state of an in-progress calibration via CDC.
/// Owned by cdc_task; passed to handlers via &mut.
#[derive(Debug)]
pub struct CalibrationSession {
    pub axis: openhotas_protocol::request::AxisId,
    pub min: Option<u16>,
    pub center: Option<u16>,
    pub max: Option<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PendingReset {
    None,
    Application,
    UsbBoot,
}

// ── Frame I/O Helpers ────────────────────────────────────────────────────

async fn send_frame(
    sender: &mut Sender<'static, Driver<'static, USB>>,
    data: &[u8],
) -> Result<(), EndpointError> {
    for chunk in data.chunks(64) {
        sender.write_packet(chunk).await?;
    }
    Ok(())
}

/// Serialize a Response into a CDC frame. Returns `Some(len)` on success,
/// `None` if the serialized payload overflows `MAX_PAYLOAD_SIZE`.
///
/// Em overflow, o caller deve responder `Response::Error(InternalError)` em
/// vez de derrubar a sessão CDC — antes, `build_frame` retornava 0 e a
/// conexão caía silenciosamente.
fn build_frame(response: &Response, buf: &mut [u8]) -> Option<usize> {
    let payload_start = 4;
    let payload_buf = &mut buf[payload_start..payload_start + MAX_PAYLOAD_SIZE];

    let serialized = postcard::to_slice(response, payload_buf).ok()?;
    let payload_len = serialized.len();

    buf[0] = SOF_A;
    buf[1] = SOF_B;
    let len_bytes = (payload_len as u16).to_be_bytes();
    buf[2] = len_bytes[0];
    buf[3] = len_bytes[1];

    let crc = openhotas_protocol::frame::crc16_ccitt(&buf[2..4 + payload_len]);
    let crc_bytes = crc.to_be_bytes();
    let crc_offset = payload_start + payload_len;
    buf[crc_offset] = crc_bytes[0];
    buf[crc_offset + 1] = crc_bytes[1];

    Some(crc_offset + 2)
}

/// Constroi e envia um frame. Retorna `false` se a conexão deve ser encerrada
/// (falha de I/O no USB), `true` para continuar a sessão.
/// Em overflow de serialização, envia `Response::Error(InternalError)` e mantém
/// a sessão aberta.
async fn send_response(
    response: &Response,
    buf: &mut [u8],
    sender: &mut Sender<'static, Driver<'static, USB>>,
) -> bool {
    let Some(len) = build_frame(response, buf) else {
        let err = Response::Error(openhotas_protocol::error::ProtocolError::InternalError);
        if let Some(len) = build_frame(&err, buf) {
            return send_frame(sender, &buf[..len]).await.is_ok();
        }
        return false;
    };
    send_frame(sender, &buf[..len]).await.is_ok()
}

// ── Request Dispatch ─────────────────────────────────────────────────────

fn handle_request(
    req: &Request,
    active_config: &mut openhotas_protocol::config::DeviceConfig,
    cal_session: &mut Option<CalibrationSession>,
    pending_reset: &mut PendingReset,
) -> Response {
    match req {
        Request::GetInfo
        | Request::GetConfig
        | Request::GetRawAxes
        | Request::GetProcessedAxes
        | Request::GetButtonStates
        | Request::GetSensorStatus
        | Request::GetRuntimeStats
        | Request::GetErrorCounters => handle_read_request(req, active_config),

        Request::SetConfig(_)
        | Request::SaveConfig
        | Request::LoadDefaults
        | Request::FactoryReset
        | Request::Reboot
        | Request::RebootToBootloader => handle_write_request(req, active_config, pending_reset),

        Request::StartCalibration(_)
        | Request::CaptureCalibrationPoint { .. }
        | Request::FinishCalibration(_) => {
            handle_calibration_request(req, active_config, cal_session)
        }
    }
}

// ── Task ─────────────────────────────────────────────────────────────────

#[embassy_executor::task]
pub async fn cdc_task(
    mut sender: Sender<'static, Driver<'static, USB>>,
    mut receiver: Receiver<'static, Driver<'static, USB>>,
) -> ! {
    let mut active_config = stored_config_v2::load_config();
    if let Some(runtime_cfg) = runtime::from_protocol_config(&active_config) {
        signal_latest_config(runtime_cfg);
    }

    let mut cal_session: Option<CalibrationSession> = None;
    let mut pending_reset = PendingReset::None;
    let mut parser = FrameParser::new();
    let mut read_buf = [0u8; 64];
    const _: () = assert!(
        4 + MAX_PAYLOAD_SIZE + 2 <= 300,
        "frame_buf overflow: MAX_PAYLOAD_SIZE muito grande"
    );
    let mut frame_buf = [0u8; 300];

    loop {
        sender.wait_connection().await;

        'connection: loop {
            let n = match receiver.read_packet(&mut read_buf).await {
                Ok(n) => n,
                Err(EndpointError::Disabled) | Err(_) => break,
            };

            for &byte in read_buf.iter().take(n) {
                match parser.feed(byte) {
                    Ok(Some(frame)) => match postcard::from_bytes::<Request>(&frame.payload) {
                        Ok(request) => {
                            let response = handle_request(
                                &request,
                                &mut active_config,
                                &mut cal_session,
                                &mut pending_reset,
                            );
                            if !send_response(&response, &mut frame_buf, &mut sender).await {
                                break 'connection;
                            }
                            if pending_reset != PendingReset::None {
                                // V1.23 (F2): 100ms delay so PC receives Ack before reset
                                Timer::after_millis(100).await;
                                match pending_reset {
                                    PendingReset::Application => {
                                        cortex_m::peripheral::SCB::sys_reset()
                                    }
                                    PendingReset::UsbBoot => {
                                        embassy_rp::rom_data::reset_to_usb_boot(0, 0)
                                    }
                                    PendingReset::None => {}
                                }
                            }
                        }
                        Err(_) => {
                            let response = Response::Error(
                                openhotas_protocol::error::ProtocolError::InvalidPayload,
                            );
                            if !send_response(&response, &mut frame_buf, &mut sender).await {
                                break 'connection;
                            }
                        }
                    },
                    Ok(None) => {}
                    Err(FrameError::CrcMismatch | FrameError::InvalidLength) => {
                        runtime_stats::PROTOCOL_CRC_ERRORS.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }

        cal_session = None;
        pending_reset = PendingReset::None;
        parser = FrameParser::new();
    }
}
