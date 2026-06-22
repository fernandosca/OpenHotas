// ─── Mirroring openhotas-protocol Rust types ───────────────────────────────
// Keep in sync with: config.rs, diagnostics.rs, response.rs, request.rs

export type AxisId = "X" | "Y" | "Twist";
export type CalibrationPoint = "Min" | "Center" | "Max";

// ── config.rs ───────────────────────────────────────────────────────────────

export interface CalibrationData {
  min_raw: number;
  center_raw: number;
  max_raw: number;
}

export interface AxisTravelLimits {
  travel_limit_pct: number; // 1..100
}

export interface CurvePoint {
  x: number; // i16 permille: -1000..1000
  y: number; // i16 permille: -1000..1000
}

export interface ResponseCurveData {
  point_left: CurvePoint;   // P1: x in (-1000, 0)
  point_right: CurvePoint;  // P3: x in (0, 1000)
}

export interface AxisConfig {
  enabled: boolean;
  inverted: boolean;
  calibration: CalibrationData;
  travel: AxisTravelLimits;
  deadzone_permille: number; // 0..200
  ema_permille: number;      // 1..1000
  max_jump_raw: number;      // 1..32767
  response_curve: ResponseCurveData;
  reset_ema_on_dz: boolean;
  axis_to_button: AxisToButtonConfig;
  center_offset_permille: number; // -200..200
}

export interface AxisToButtonConfig {
  enabled: boolean;
  threshold_permille: number; // 0..1000
  direction: "Positive" | "Negative" | "Both";
  button_index: number; // 0..31
}

export interface ButtonConfig {
  enabled_mask: number;  // u32 bitmask
  inverted_mask: number; // u32 bitmask
  debounce_ms: number;   // 1 | 2 | 5 | 10 | 20
}

export interface DeviceConfig {
  protocol_version_major: number;
  protocol_version_minor: number;
  axes: [AxisConfig, AxisConfig, AxisConfig]; // [X, Y, Twist]
  buttons: ButtonConfig;
}

// ── diagnostics.rs ──────────────────────────────────────────────────────────

export interface RuntimeStats {
  reports_sent: number;
  send_errors: number;
  sensor_cycles: number;
  last_cycle_us: number;
  max_cycle_us: number;
}

export interface RawAxes {
  x: number;  // u16
  y: number;
  twist: number;
}

export interface ProcessedAxes {
  x: number;     // i16 [-32767, 32767]
  y: number;
  twist: number;
  unhealthy_mask: number; // bit 0=X, bit 1=Y, bit 2=Twist
}

export interface ButtonStates {
  mask: number; // u32
}

export interface SensorInfo {
  healthy: boolean;
  error_count: number;
}

export interface SensorStatusReport {
  x: SensorInfo;
  y: SensorInfo;
  twist: SensorInfo;
}

export interface ErrorCounters {
  protocol_crc_errors: number;
  sensor_crc_errors: number;
  magnet_errors: number;
  flash_errors: number;
  button_errors: number;
  buttons_degraded: boolean;
}

// ── response.rs ─────────────────────────────────────────────────────────────

export interface DeviceInfo {
  firmware_version: number[]; // [u8; 8] — parse as UTF-8 string
  git_hash: number[];         // [u8; 8] — 7-char short hash
  protocol_major: number;
  protocol_minor: number;
  axis_count: number;
  button_count: number;
}

// Tagged union matching the Rust Response enum.
// Tauri serializes enums as { variant: "...", ...fields } by default.
export type Response =
  | { variant: "Ack" }
  | { variant: "Error"; error: ProtocolError }
  | { variant: "Info"; info: DeviceInfo }
  | { variant: "Config"; config: DeviceConfig }
  | { variant: "RawAxes"; raw_axes: RawAxes }
  | { variant: "ProcessedAxes"; processed_axes: ProcessedAxes }
  | { variant: "ButtonStates"; button_states: ButtonStates }
  | { variant: "SensorStatus"; sensor_status: SensorStatusReport }
  | { variant: "RuntimeStats"; runtime_stats: RuntimeStats }
  | { variant: "ErrorCounters"; error_counters: ErrorCounters };

// ── error.rs ────────────────────────────────────────────────────────────────

export type ProtocolError =
  | "UnknownCommand"
  | "InvalidPayload"
  | "InvalidLength"
  | "CrcMismatch"
  | "UnsupportedVersion"
  | "Busy"
  | "FlashError"
  | "InvalidConfig"
  | "CalibrationError"
  | "InternalError";

// ── helpers ─────────────────────────────────────────────────────────────────

export const AXIS_INDEX: Record<AxisId, 0 | 1 | 2> = { X: 0, Y: 1, Twist: 2 };

export function firmwareVersionString(fw: number[]): string {
  return fw
    .filter((b) => b !== 0)
    .map((b) => String.fromCharCode(b))
    .join("");
}

export function gitHashString(hash: number[]): string {
  return hash
    .filter((b) => b !== 0)
    .map((b) => String.fromCharCode(b))
    .join("");
}

export function isAxisHealthy(mask: number, axis: AxisId): boolean {
  const bit = AXIS_INDEX[axis];
  return (mask & (1 << bit)) === 0;
}

export function defaultAxisConfig(): AxisConfig {
  return {
    enabled: true,
    inverted: false,
    calibration: { min_raw: 0, center_raw: 16384, max_raw: 32767 },
    travel: { travel_limit_pct: 100 },
    deadzone_permille: 20,
    ema_permille: 300,
    max_jump_raw: 4915,
    response_curve: {
      point_left: { x: -500, y: -500 },
      point_right: { x: 500, y: 500 },
    },
    reset_ema_on_dz: false,
    axis_to_button: {
      enabled: false,
      threshold_permille: 800,
      direction: "Both",
      button_index: 0,
    },
    center_offset_permille: 0,
  };
}

export function defaultDeviceConfig(): DeviceConfig {
  const axes: [AxisConfig, AxisConfig, AxisConfig] = [
    defaultAxisConfig(),
    defaultAxisConfig(),
    { ...defaultAxisConfig(), reset_ema_on_dz: true }, // Twist
  ];
  return {
    protocol_version_major: 3,
    protocol_version_minor: 0,
    axes,
    buttons: {
      enabled_mask: 0xffffffff,
      inverted_mask: 0xffffffff,
      debounce_ms: 5,
    },
  };
}
