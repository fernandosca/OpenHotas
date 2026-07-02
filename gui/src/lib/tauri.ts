/**
 * Typed wrappers around Tauri IPC commands.
 *
 * Each function maps 1-to-1 to a #[tauri::command] in src-tauri/src/commands.rs.
 * The backend sends/receives postcard frames via the CDC serial port; these
 * wrappers hide that completely from the UI.
 */

import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";
import { defaultDeviceConfig } from "../types/protocol";
import type {
  DeviceConfig,
  DeviceInfo,
  ProcessedAxes,
  RawAxes,
  ButtonStates,
  SensorStatusReport,
  RuntimeStats,
  ErrorCounters,
  AxisId,
  CalibrationPoint,
} from "../types/protocol";

const browserStartedAt = Date.now();

function mockAxes(): ProcessedAxes {
  const t = (Date.now() - browserStartedAt) / 1000;
  return {
    x: Math.round(Math.sin(t * 1.7) * 24000),
    y: Math.round(Math.cos(t * 1.3) * 22000),
    twist: Math.round(Math.sin(t * 0.9) * 18000),
    unhealthy_mask: 0,
  };
}

function mockRawAxes(): RawAxes {
  const axes = mockAxes();
  return {
    x: Math.round(((axes.x + 32767) / 65534) * 32767),
    y: Math.round(((axes.y + 32767) / 65534) * 32767),
    twist: Math.round(((axes.twist + 32767) / 65534) * 32767),
  };
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(command, args);
  }

  switch (command) {
    case "list_ports":
      return [{ name: "BROWSER_PREVIEW", description: "Preview mock" }] as T;
    case "connect":
    case "get_info":
      return {
        firmware_version: [49, 46, 50, 46, 51, 0, 0, 0],
        git_hash: [112, 114, 101, 118, 105, 101, 119, 0],
        protocol_major: 1,
        protocol_minor: 0,
        axis_count: 3,
        button_count: 32,
      } as T;
    case "get_config":
      return defaultDeviceConfig() as T;
    case "get_raw_axes":
      return mockRawAxes() as T;
    case "get_processed_axes":
      return mockAxes() as T;
    case "get_button_states":
      return { mask: Math.floor(Date.now() / 500) % 2 ? 0x00000005 : 0 } as T;
    case "get_sensor_status":
      return {
        x: { healthy: true, error_count: 0 },
        y: { healthy: true, error_count: 0 },
        twist: { healthy: true, error_count: 0 },
      } as T;
    case "get_runtime_stats":
      return {
        reports_sent: Math.floor((Date.now() - browserStartedAt) / 16),
        send_errors: 0,
        sensor_cycles: Math.floor((Date.now() - browserStartedAt) / 1000),
        last_cycle_us: 420,
        max_cycle_us: 610,
      } as T;
    case "get_error_counters":
      return {
        protocol_crc_errors: 0,
        sensor_crc_errors: 0,
        magnet_errors: 0,
        flash_errors: 0,
        button_errors: 0,
        buttons_degraded: false,
      } as T;
    default:
      return undefined as T;
  }
}

// ── System ──────────────────────────────────────────────────────────────────

/** Query device identity and protocol version. Always first call after connect. */
export async function getInfo(): Promise<DeviceInfo> {
  return invoke<DeviceInfo>("get_info");
}

/** Software reboot — device stays in application mode. */
export async function reboot(): Promise<void> {
  return invoke("reboot");
}

/** Erase config + calibration, reboot with factory defaults. */
export async function factoryReset(): Promise<void> {
  return invoke("factory_reset");
}

// ── Configuration ────────────────────────────────────────────────────────────

/** Read current active (runtime) configuration. */
export async function getConfig(): Promise<DeviceConfig> {
  return invoke<DeviceConfig>("get_config");
}

/**
 * Apply new configuration at runtime (NOT persisted to flash).
 * Call saveConfig() afterwards to persist.
 */
export async function setConfig(config: DeviceConfig): Promise<void> {
  return invoke("set_config", { config });
}

/** Persist current runtime config to flash. */
export async function saveConfig(): Promise<void> {
  return invoke("save_config");
}

/** Reload firmware defaults into runtime config (does NOT write flash). */
export async function loadDefaults(): Promise<void> {
  return invoke("load_defaults");
}

// ── Diagnostics ──────────────────────────────────────────────────────────────

export async function getRawAxes(): Promise<RawAxes> {
  return invoke<RawAxes>("get_raw_axes");
}

export async function getProcessedAxes(): Promise<ProcessedAxes> {
  return invoke<ProcessedAxes>("get_processed_axes");
}

export async function getButtonStates(): Promise<ButtonStates> {
  return invoke<ButtonStates>("get_button_states");
}

export async function getSensorStatus(): Promise<SensorStatusReport> {
  return invoke<SensorStatusReport>("get_sensor_status");
}

export async function getRuntimeStats(): Promise<RuntimeStats> {
  return invoke<RuntimeStats>("get_runtime_stats");
}

export async function getErrorCounters(): Promise<ErrorCounters> {
  return invoke<ErrorCounters>("get_error_counters");
}

// ── Calibration ───────────────────────────────────────────────────────────────

/** Open calibration session for axis. Must call before capturing points. */
export async function startCalibration(axis: AxisId): Promise<void> {
  return invoke("start_calibration", { axis });
}

/**
 * Capture a single calibration point (Min / Center / Max).
 * Move axis to position BEFORE calling this.
 */
export async function captureCalibrationPoint(
  axis: AxisId,
  point: CalibrationPoint
): Promise<void> {
  return invoke("capture_calibration_point", { axis, point });
}

/**
 * Finish calibration — applies new cal to runtime config.
 * Call saveConfig() to persist to flash.
 */
export async function finishCalibration(axis: AxisId): Promise<void> {
  return invoke("finish_calibration", { axis });
}

// ── Serial port discovery (custom command, not in protocol) ──────────────────

export interface PortInfo {
  name: string;
  description: string | null;
}

/** List available serial ports (for device selection UI). */
export async function listPorts(): Promise<PortInfo[]> {
  return invoke<PortInfo[]>("list_ports");
}

/** Connect to device on the given port. */
export async function connect(portName: string): Promise<DeviceInfo> {
  return invoke<DeviceInfo>("connect", { portName });
}

/** Disconnect from current device. */
export async function disconnect(): Promise<void> {
  return invoke("disconnect");
}

export interface FirmwareUpdateResult {
  volume: string;
  bytes_copied: number;
}

/** Reboot the connected device into ROM boot mode and copy a validated UF2. */
export async function installFirmware(path: string): Promise<FirmwareUpdateResult> {
  return invoke<FirmwareUpdateResult>("install_firmware", { path });
}
