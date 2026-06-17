#![allow(dead_code)]

//! # OpenHOTAS — Fonte Unica de Constantes
//!
//! Todas as constantes do projeto estao aqui.
//! Use: `use crate::constants::*;`
//! NUNCA redefinir constantes localmente nos modulos.
//!
//! ARQUIVO DE CONTRATO DE HARDWARE — NAO FRAGMENTAR
//!
//! Organizacao:
//!   - Hardware (MT6826S, MCP23S17): contrato fixo, nao alterar sem re-testar
//!   - Eixos, Flash, USB HID: estruturais, estaveis
//!   - `pub mod tuning`: valores ajustaveis em campo — separados por design

// ── Hardware: MT6826S (SPI1) ─────────────────────────────────────────────
// VALIDADAS no firmware V1.1 (Datasheet Rev.1.1) — nao alterar sem re-testar

pub const MT6826_SPI_FREQ_HZ: u32 = 1_000_000;

pub const MT6826_SPI_MODE: u8 = 3;

/// Comando de leitura de angulo — Burst Angle Read (datasheet §8.6.8)
/// Frame de comando: C3-C0 = 1010 (0x0A)
pub const MT6826_CMD_READ_ANGLE: u8 = 0x0A;

pub const MT6826_CRC8_POLY: u8 = 0x07;

pub const MT6826_MAGNET_OK_MASK: u8 = 0x06;

pub const MT6826_ANGLE_SHIFT: u8 = 1;

pub const MT6826_ANGLE_MAX: u16 = 32767;

pub const MT6826_ANGLE_CENTER: u16 = 16384;

// ── Hardware: MCP23S17 (SPI0) ────────────────────────────────────────────

pub const MCP23S17_DEBOUNCE_COUNT: u8 = 3;

pub const MCP23S17_IODIRA: u8 = 0x00;
pub const MCP23S17_IODIRB: u8 = 0x01;
pub const MCP23S17_IOCON: u8 = 0x0A;
pub const MCP23S17_GPPUA: u8 = 0x0C;
pub const MCP23S17_GPPUB: u8 = 0x0D;
pub const MCP23S17_GPIOA: u8 = 0x12;
pub const MCP23S17_GPIOB: u8 = 0x13;

// ── Eixos ────────────────────────────────────────────────────────────────

pub const AXIS_COUNT: usize = 3;

pub const AXIS_X: usize = 0;

pub const AXIS_Y: usize = 1;

pub const AXIS_TWIST: usize = 2;

// ── Flash ────────────────────────────────────────────────────────────────
//
// Estes valores sao OFFSETS relativos ao inicio da flash fisica (0x00),
// NAO enderecos XIP absolutos (base XIP do RP2350 = 0x10000000).
//
// Para operacoes de erase/write via embassy-rp:
//   usar o offset diretamente
//
// Para leitura via memory-mapped XIP (ponteiro), use:
//   let ptr = (0x10000000u32 + CALIB_OFFSET) as *const u8;

pub const FLASH_SIZE: u32 = 2 * 1024 * 1024;

pub const SECTOR_SIZE: u32 = 4096;

pub const CONFIG_OFFSET: u32 = FLASH_SIZE - SECTOR_SIZE;

pub const CALIB_OFFSET: u32 = CONFIG_OFFSET - SECTOR_SIZE;

pub const MAGIC_DEVICE: u32 = 0x484F5441;

pub const MAGIC_CAL: u32 = 0x43414C31;

pub const CONFIG_VERSION: u8 = 1;

// ── USB HID ──────────────────────────────────────────────────────────────

pub const REPORT_ID_GAMEPAD: u8 = 0x01;

pub const REPORT_ID_CONFIG: u8 = 0x02;

pub const REPORT_SIZE: usize = 10;

pub const HID_AXIS_MAX: i16 = 32767;

// ── Diagnostico ──────────────────────────────────────────────────────────

pub const DIAGNOSTIC_INTERVAL_SECS: u64 = 5;

pub const MAX_INPUT_CYCLE_US: u32 = 500;

// ── Versionamento do Firmware ───────────────────────────────────────────

/// Versão SemVer do firmware — lida do Cargo.toml em tempo de compilação
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git hash curto do commit que gerou este binário
pub const FIRMWARE_GIT_HASH: &str = env!("GIT_HASH");

// ── Tuning Layer ─────────────────────────────────────────────────────────
//
// Constantes ajustaveis em campo, separadas das constantes de hardware por
// design. Representam defaults que podem ser sobrescritos via calibracao
// salva em flash (DeviceConfig). Alterar aqui afeta apenas o comportamento
// fora de calibracao — NAO afeta o contrato de hardware acima.

pub mod tuning {
    pub const DEFAULT_EMA_ALPHA: f32 = 0.3;

    pub const DEFAULT_DEADZONE: f32 = 0.02;

    pub const DEFAULT_MAX_JUMP: f32 = 0.15;

    pub const DEFAULT_EXPO: f32 = 0.0;
}
