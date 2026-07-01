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

// ── Pin Assignments (GPIO) — Referência Documental ─────────────────────
// Embassy usa type-level pins (p.PIN_X), então estas constantes são
// apenas para referência. Os valores reais estão em main.rs.

// SPI0 — MCP23S17 (botões)
pub const PIN_SPI0_SCK: u8 = 6;
pub const PIN_SPI0_MOSI: u8 = 7;
pub const PIN_SPI0_MISO: u8 = 4;
pub const PIN_CS_MCP23S: u8 = 5;
pub const PIN_INT_MCP23S: u8 = 8; // Wired-OR INT de U1+U2 (não implementado, apenas referência)

// SPI1 — MT6826S (eixos)
pub const PIN_SPI1_SCK: u8 = 14;
pub const PIN_SPI1_MOSI: u8 = 15;
pub const PIN_SPI1_MISO: u8 = 12;
pub const PIN_CS_ENCODER_X: u8 = 10;
pub const PIN_CS_ENCODER_Y: u8 = 13;
pub const PIN_CS_ENCODER_TWIST: u8 = 16;

// ── Hardware: MT6826S (SPI1) ─────────────────────────────────────────────
// VALIDADAS no firmware V1.1 (Datasheet Rev.1.1) — nao alterar sem re-testar

pub const MT6826_SPI_FREQ_HZ: u32 = 1_000_000;

pub const MT6826_SPI_MODE: u8 = 3;

/// Tempo entre CSN baixo e o primeiro clock.
/// Datasheet: TL >= 100 ns. Um microssegundo fornece margem ampla.
pub const MT6826_CS_SETUP_US: u64 = 1;

/// Tempo entre o ultimo rising edge de SCK e CSN alto.
/// Datasheet: TH >= 0,5 * TSCK. Em 1 MHz, TH >= 0,5 us; use 1 us de margem.
pub const MT6826_CS_HOLD_US: u64 = 1;

/// Tempo de inicializacao do MT6826S apos energizacao.
/// Datasheet: TPwrUp tipico de 3 ms. Use margem antes da primeira leitura.
pub const MT6826_POWER_UP_MS: u64 = 5;

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
//   let ptr = (0x10000000u32 + offset) as *const u8;

pub const FLASH_SIZE: u32 = 2 * 1024 * 1024;

pub const SECTOR_SIZE: u32 = 4096;

/// V1.23: StoredConfigV2 — fonte oficial de config + calibração.
/// Último setor da flash (0x001FF000).
pub const STORED_V2_OFFSET: u32 = FLASH_SIZE - SECTOR_SIZE;

// ── USB HID ──────────────────────────────────────────────────────────────

pub const REPORT_ID_GAMEPAD: u8 = 0x01;

pub const REPORT_SIZE: usize = 10;

pub const HID_AXIS_MAX: i16 = 32767;

// ── Diagnostico ──────────────────────────────────────────────────────────

pub const DIAGNOSTIC_INTERVAL_SECS: u64 = 5;

pub const MAX_INPUT_CYCLE_US: u32 = 500;

// ── Versionamento do Firmware ───────────────────────────────────────────

/// Versão SemVer do firmware — lida do Cargo.toml em tempo de compilação
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git hash curto do commit que gerou este binário
pub const FIRMWARE_GIT_HASH: &str = match option_env!("GIT_HASH") {
    Some(h) => h,
    None => "unknown",
};

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
}
