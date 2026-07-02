//! Tauri commands — bridge between the React frontend and the CDC serial port.
//!
//! Each `#[tauri::command]` maps 1-to-1 to a function in `src/lib/tauri.ts`.
//!
//! Flow:
//!   Frontend invoke() → command here → encode Request → write CDC frame →
//!   read CDC frame → decode Response → return to frontend.
//!
//! The firmware speaks the openhotas-protocol (postcard + CRC16-CCITT frames).
//! We use the `openhotas-protocol` crate directly so types are never duplicated.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use openhotas_protocol::{
    config::DeviceConfig,
    diagnostics::{
        ButtonStates, ErrorCounters, ProcessedAxes, RawAxes, RuntimeStats, SensorStatusReport,
    },
    frame::{crc16_ccitt, FrameParser, MAX_PAYLOAD_SIZE, SOF_A, SOF_B},
    request::{AxisId, CalibrationPoint, Request},
    response::{DeviceInfo, Response},
};
use serialport::SerialPort;
use tauri::State;

// ── State ────────────────────────────────────────────────────────────────────

pub struct DeviceState {
    port: Mutex<Option<Box<dyn SerialPort>>>,
    parser: Mutex<FrameParser>,
}

impl DeviceState {
    pub fn new() -> Self {
        Self {
            port: Mutex::new(None),
            parser: Mutex::new(FrameParser::new()),
        }
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct CommandError(String);

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type CmdResult<T> = Result<T, CommandError>;

// ── Frame encode/decode helpers ───────────────────────────────────────────────

fn encode_request(req: &Request) -> CmdResult<Vec<u8>> {
    let payload = postcard::to_stdvec(req).map_err(|e| CommandError(format!("serialize: {e}")))?;
    let len = payload.len() as u16;
    let len_bytes = len.to_be_bytes();

    let mut crc_input = Vec::with_capacity(2 + payload.len());
    crc_input.extend_from_slice(&len_bytes);
    crc_input.extend_from_slice(&payload);
    let crc = crc16_ccitt(&crc_input);

    let mut frame = Vec::with_capacity(4 + payload.len() + 2);
    frame.push(SOF_A);
    frame.push(SOF_B);
    frame.extend_from_slice(&len_bytes);
    frame.extend_from_slice(&payload);
    frame.extend_from_slice(&crc.to_be_bytes());
    Ok(frame)
}

fn send_recv(state: &DeviceState, req: Request) -> CmdResult<Response> {
    let mut port_guard = state.port.lock().unwrap();
    let port = port_guard
        .as_mut()
        .ok_or(CommandError::from("Not connected"))?;

    let frame = encode_request(&req)?;
    port.write_all(&frame)
        .map_err(|e| CommandError(format!("write: {e}")))?;

    // Read response with timeout
    let mut parser = state.parser.lock().unwrap();
    let mut buf = [0u8; MAX_PAYLOAD_SIZE + 8];
    let deadline = std::time::Instant::now() + Duration::from_millis(500);

    loop {
        if std::time::Instant::now() > deadline {
            return Err(CommandError::from("Timeout waiting for response"));
        }
        let n = port.read(&mut buf).unwrap_or(0);
        for &byte in &buf[..n] {
            if let Ok(Some(frame)) = parser.feed(byte) {
                let response: Response = postcard::from_bytes(&frame.payload)
                    .map_err(|e| CommandError(format!("deserialize: {e}")))?;
                if let Response::Error(e) = response {
                    return Err(CommandError(format!("firmware error: {e:?}")));
                }
                return Ok(response);
            }
        }
    }
}

// ── Port commands ─────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct PortInfo {
    name: String,
    description: Option<String>,
}

#[tauri::command]
pub fn list_ports() -> CmdResult<Vec<PortInfo>> {
    serialport::available_ports()
        .map(|ports| {
            ports
                .into_iter()
                .map(|p| PortInfo {
                    name: p.port_name,
                    description: match p.port_type {
                        serialport::SerialPortType::UsbPort(info) => info.product,
                        _ => None,
                    },
                })
                .collect()
        })
        .map_err(|e| CommandError(format!("list_ports: {e}")))
}

#[tauri::command]
pub fn connect(port_name: String, state: State<DeviceState>) -> CmdResult<DeviceInfo> {
    let port = serialport::new(&port_name, 115_200)
        .timeout(Duration::from_millis(500))
        .open()
        .map_err(|e| CommandError(format!("open {port_name}: {e}")))?;

    *state.port.lock().unwrap() = Some(port);
    *state.parser.lock().unwrap() = FrameParser::new();

    match send_recv(&state, Request::GetInfo)? {
        Response::Info(info) => Ok(info),
        other => Err(CommandError(format!("unexpected response: {other:?}"))),
    }
}

#[tauri::command]
pub fn disconnect(state: State<DeviceState>) {
    *state.port.lock().unwrap() = None;
}

// ── System commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_info(state: State<DeviceState>) -> CmdResult<DeviceInfo> {
    match send_recv(&state, Request::GetInfo)? {
        Response::Info(i) => Ok(i),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn reboot(state: State<DeviceState>) -> CmdResult<()> {
    send_recv(&state, Request::Reboot)?;
    Ok(())
}

#[tauri::command]
pub fn factory_reset(state: State<DeviceState>) -> CmdResult<()> {
    send_recv(&state, Request::FactoryReset)?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct FirmwareUpdateResult {
    volume: String,
    bytes_copied: u64,
}

fn validate_uf2(path: &Path) -> CmdResult<u64> {
    if path
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.eq_ignore_ascii_case("uf2"))
        != Some(true)
    {
        return Err(CommandError::from("Select a .uf2 file"));
    }
    let data = fs::read(path).map_err(|e| CommandError(format!("read UF2: {e}")))?;
    if data.is_empty() || data.len() % 512 != 0 {
        return Err(CommandError::from(
            "Invalid UF2: size must be a non-zero multiple of 512 bytes",
        ));
    }
    for block in data.chunks_exact(512) {
        let u32_at = |offset| u32::from_le_bytes(block[offset..offset + 4].try_into().unwrap());
        if u32_at(0) != 0x0A32_4655 || u32_at(4) != 0x9E5D_5157 || u32_at(508) != 0x0AB1_6F30 {
            return Err(CommandError::from("Invalid UF2 block signature"));
        }
    }
    Ok(data.len() as u64)
}

#[cfg(target_os = "windows")]
fn boot_volumes() -> Vec<PathBuf> {
    (b'D'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", letter as char)))
        .filter(|root| root.join("INFO_UF2.TXT").is_file())
        .collect()
}

#[cfg(target_os = "macos")]
fn boot_volumes() -> Vec<PathBuf> {
    fs::read_dir("/Volumes")
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|root| root.join("INFO_UF2.TXT").is_file())
        .collect()
}

#[cfg(target_os = "linux")]
fn boot_volumes() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/media"),
        PathBuf::from("/run/media"),
        PathBuf::from("/mnt"),
    ];
    let mut candidates = Vec::new();
    while let Some(root) = roots.pop() {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("INFO_UF2.TXT").is_file() {
                candidates.push(path);
            } else if path.is_dir() {
                roots.push(path);
            }
        }
    }
    candidates
}

#[tauri::command]
pub fn install_firmware(
    path: String,
    state: State<DeviceState>,
) -> CmdResult<FirmwareUpdateResult> {
    let source = PathBuf::from(path);
    let expected_size = validate_uf2(&source)?;
    let volumes_before = boot_volumes();

    match send_recv(&state, Request::RebootToBootloader)? {
        Response::Ack => {}
        other => return Err(CommandError(format!("unexpected: {other:?}"))),
    }
    // Release the serial handle before Windows enumerates the ROM bootloader.
    *state.port.lock().unwrap() = None;

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let volume = loop {
        let candidates = boot_volumes();
        let mut new_candidates: Vec<_> = candidates
            .iter()
            .filter(|path| !volumes_before.contains(path))
            .cloned()
            .collect();
        if new_candidates.len() == 1 {
            break new_candidates.remove(0);
        }
        // A Pico may already be in boot mode; accept it only when unambiguous.
        if volumes_before.is_empty() && candidates.len() == 1 {
            break candidates[0].clone();
        }
        if std::time::Instant::now() >= deadline {
            return Err(CommandError::from(
                "RPI-RP2 volume not found within 15 seconds",
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    };

    let destination = volume.join("openhotas.uf2");
    let bytes_copied = fs::copy(&source, &destination)
        .map_err(|e| CommandError(format!("copy UF2 to {}: {e}", volume.display())))?;
    if bytes_copied != expected_size {
        return Err(CommandError::from("Incomplete UF2 copy"));
    }
    Ok(FirmwareUpdateResult {
        volume: volume.display().to_string(),
        bytes_copied,
    })
}

// ── Config commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_config(state: State<DeviceState>) -> CmdResult<DeviceConfig> {
    match send_recv(&state, Request::GetConfig)? {
        Response::Config(c) => Ok(c),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn set_config(config: DeviceConfig, state: State<DeviceState>) -> CmdResult<()> {
    match send_recv(&state, Request::SetConfig(config))? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn save_config(state: State<DeviceState>) -> CmdResult<()> {
    match send_recv(&state, Request::SaveConfig)? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn load_defaults(state: State<DeviceState>) -> CmdResult<()> {
    match send_recv(&state, Request::LoadDefaults)? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

// ── Diagnostic commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_raw_axes(state: State<DeviceState>) -> CmdResult<RawAxes> {
    match send_recv(&state, Request::GetRawAxes)? {
        Response::RawAxes(a) => Ok(a),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn get_processed_axes(state: State<DeviceState>) -> CmdResult<ProcessedAxes> {
    match send_recv(&state, Request::GetProcessedAxes)? {
        Response::ProcessedAxes(a) => Ok(a),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn get_button_states(state: State<DeviceState>) -> CmdResult<ButtonStates> {
    match send_recv(&state, Request::GetButtonStates)? {
        Response::ButtonStates(b) => Ok(b),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn get_sensor_status(state: State<DeviceState>) -> CmdResult<SensorStatusReport> {
    match send_recv(&state, Request::GetSensorStatus)? {
        Response::SensorStatus(s) => Ok(s),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn get_runtime_stats(state: State<DeviceState>) -> CmdResult<RuntimeStats> {
    match send_recv(&state, Request::GetRuntimeStats)? {
        Response::RuntimeStats(s) => Ok(s),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn get_error_counters(state: State<DeviceState>) -> CmdResult<ErrorCounters> {
    match send_recv(&state, Request::GetErrorCounters)? {
        Response::ErrorCounters(e) => Ok(e),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

// ── Calibration commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn start_calibration(axis: AxisId, state: State<DeviceState>) -> CmdResult<()> {
    match send_recv(&state, Request::StartCalibration(axis))? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn capture_calibration_point(
    axis: AxisId,
    point: CalibrationPoint,
    state: State<DeviceState>,
) -> CmdResult<()> {
    match send_recv(&state, Request::CaptureCalibrationPoint { axis, point })? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}

#[tauri::command]
pub fn finish_calibration(axis: AxisId, state: State<DeviceState>) -> CmdResult<()> {
    match send_recv(&state, Request::FinishCalibration(axis))? {
        Response::Ack => Ok(()),
        other => Err(CommandError(format!("unexpected: {other:?}"))),
    }
}
