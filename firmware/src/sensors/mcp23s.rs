//! Driver para MCP23S17 — expansor GPIO via SPI (botões do stick).
//!
//! # Hardware
//!
//! Dois MCP23S17 no barramento SPI0, endereçados por hardware:
//! - U1: ADDR=0 (pino ADDR0=GND) → GPIOA/B (16 bits, botões físicos)
//! - U2: ADDR=1 (pino ADDR0=VCC) → GPIOA/B (16 bits, botões físicos)
//!
//! Total: 32 botões, mapeados como bits 0..31 no u32 de saída.
//! bit 0 = U1.GPIOA.0, bit 15 = U1.GPIOB.7, bit 16 = U2.GPIOA.0, ...
//!
//! Os chips são configurados com pull-ups internos habilitados e todas
//! as portas como entrada. Botões conectam o pino ao GND quando pressionados
//! (active low). O estado "liberado" é lido como 1 (pull-up), "pressionado" como 0.
//! A saída do driver já normaliza para 0xFFFF_FFFF = todos liberados.
//!
//! # Modos de falha
//!
//! - SPI timeout: não há timeout na HAL blocking — se MISO travar, o chip
//!   não responde e o barramento fica preso (ver `spi_bus`).
//! - MISO preso em 0x00: a leitura retorna 0x0000 sem erro — botões aparecem
//!   como "todos pressionados". A verificação `init_chips` detecta no boot,
//!   mas se a falha ocorrer em operação, não há detecção.
//! - MCP ausente: detectado pelo readback no `init_chips`. Se a falha ocorrer
//!   após o init, o driver retorna `MCP23S17_BUTTONS_RELEASED` (perda total
//!   dos botões, sem alarme no HID).
//!
//! # no_std / heap
//!
//! Sem heap. Toda I/O é SPI blocking. CS é pino GPIO — toggle manual para
//! controle fino de timing.

use super::{Sensor, SensorError, SensorHealth};
use crate::constants::{
    MCP23S17_DEBOUNCE_COUNT, MCP23S17_GPIOA, MCP23S17_GPPUA, MCP23S17_GPPUB, MCP23S17_IOCON,
    MCP23S17_IODIRA, MCP23S17_IODIRB,
};
use crate::spi_bus;
use embassy_rp::gpio::Output;

// Endereço físico dos chips MCP23S17 no barramento SPI0.
// Determinado pelo nível do pino ADDR0 na PCB:
//   CHIP_ADDR_U1 = 0x00  (ADDR0 = GND)
//   CHIP_ADDR_U2 = 0x01  (ADDR0 = VCC)
const CHIP_ADDR_U1: u8 = 0x00;
const CHIP_ADDR_U2: u8 = 0x01;

/// Intervalo (em ciclos de `read()`) entre verificações de saúde do MCP.
/// 2000 ciclos a 500μs ≈ 1 segundo. Suficiente para detectar falha pós-boot
/// sem adicionar latência significativa ao ciclo de input.
const HEALTH_CHECK_INTERVAL: u32 = 2000;

/// Máscara que representa todos os botões liberados (32 bits em nível alto).
/// MCP23S17 tem pull-ups internos; pino não pressionado = 1.
/// O merge de dois chips de 16 bits resulta em 0xFFFF_FFFF.
const MCP23S17_BUTTONS_RELEASED: u32 = 0xFFFF_FFFF;

/// Monta o opcode de escrita para o MCP23S17.
/// Formato: 0 1 0 0 A A A R, onde A=endereço do chip, R=0 (escrita).
fn write_opcode(addr: u8) -> u8 {
    0x40 | (addr << 1)
}

/// Monta o opcode de leitura para o MCP23S17.
/// Formato: 0 1 0 0 A A A R, onde A=endereço do chip, R=1 (leitura).
fn read_opcode(addr: u8) -> u8 {
    0x41 | (addr << 1)
}

/// Estado interno de debounce para um chip MCP23S17.
///
/// Implementa um filtro por contagem de leituras consecutivas iguais.
/// A saída (`state`) só muda após N leituras idênticas (`threshold`).
/// Isso rejeita ruído de bouncing dos botões mecânicos.
///
/// Invariante: `state` e `raw_prev` inicializam em 0xFFFF (todos liberados).
/// Se ocorrer power loss no meio de uma sequência de debounce, o estado
/// é perdido e recomeça — aceitável para botões.
#[derive(Debug)]
struct ChipState {
    /// Valor debounced atual (só muda após `threshold` leituras iguais).
    state: u16,
    /// Último valor lido (antes do debounce).
    raw_prev: u16,
    /// Leituras consecutivas em que `raw_prev` se manteve estável.
    stable_cnt: u8,
}

impl ChipState {
    fn new() -> Self {
        Self {
            state: 0xFFFF,
            raw_prev: 0xFFFF,
            stable_cnt: 0,
        }
    }
}

#[derive(Debug)]
pub struct Mcp23s<'d> {
    cs: Output<'d>,
    chip0: ChipState,
    chip1: ChipState,
    error_count: u32,
    /// Número de leituras consecutivas iguais exigidas para considerar estável.
    /// Default: MCP23S17_DEBOUNCE_COUNT. Configurável via ButtonRuntimeConfig.
    debounce_threshold: u8,
    /// `true` se ambos os chips foram inicializados com sucesso.
    available: bool,
    /// Current health status — updated on each `read()` and periodic IOCON check.
    /// `Failed` means bus-level error (SPI not initialized or both chips dead).
    health: SensorHealth,
    /// Cycle counter for periodic runtime health check.
    /// IOCON is read back every HEALTH_CHECK_INTERVAL cycles to detect
    /// chips that died after boot.
    cycle_count: u32,
}

impl<'d> Mcp23s<'d> {
    pub fn new(cs: Output<'d>) -> Self {
        Self {
            cs,
            chip0: ChipState::new(),
            chip1: ChipState::new(),
            error_count: 0,
            debounce_threshold: MCP23S17_DEBOUNCE_COUNT,
            available: false,
            health: SensorHealth::Healthy,
            cycle_count: 0,
        }
    }

    /// Inicializa ambos os chips e marca `available`.
    /// Se falhar, `available` fica `false` e o `read()` retorna todos liberados.
    pub fn init(&mut self) -> Result<(), SensorError> {
        let result = self.init_chips();
        self.available = result.is_ok();
        result
    }

    /// Configura registradores de ambos os chips e verifica por readback.
    ///
    /// IOCON = 0x0C: HW por HW address (bit 1=0), Sequential Operation (bit 0=0 → desligado?
    ///   Na verdade 0x0C = 0000_1100: HAEN=1 (habilita hardware addr), DISSLW=1 (desabilita slew rate).
    ///   Sequential Operation default é ligado — mas não é alterado.
    /// IODIR = 0xFF: todas as portas como entrada.
    /// GPPU = 0xFF: pull-ups internos habilitados em todas as portas.
    fn init_chips(&mut self) -> Result<(), SensorError> {
        for addr in [CHIP_ADDR_U1, CHIP_ADDR_U2] {
            self.write_reg(addr, MCP23S17_IOCON, 0x0C)?;
            self.write_reg(addr, MCP23S17_IODIRA, 0xFF)?;
            self.write_reg(addr, MCP23S17_IODIRB, 0xFF)?;
            self.write_reg(addr, MCP23S17_GPPUA, 0xFF)?;
            self.write_reg(addr, MCP23S17_GPPUB, 0xFF)?;

            // SPI writes cannot prove that a chip is present. Read back one
            // configured register from each group so a missing/floating MISO
            // cannot be reported as a healthy button bus.
            // Gap: readback só ocorre no boot. Se um chip morrer em operação,
            // o driver não detecta e retorna dados espúrios.
            let verified = self.read_reg(addr, MCP23S17_IOCON)? == 0x0C
                && self.read_reg(addr, MCP23S17_IODIRA)? == 0xFF
                && self.read_reg(addr, MCP23S17_IODIRB)? == 0xFF
                && self.read_reg(addr, MCP23S17_GPPUA)? == 0xFF
                && self.read_reg(addr, MCP23S17_GPPUB)? == 0xFF;
            if !verified {
                self.error_count = self.error_count.saturating_add(1);
                return Err(SensorError::NotPresent);
            }
        }
        Ok(())
    }

    /// Escreve em um registrador de um chip.
    /// Em caso de erro SPI, incrementa `error_count`, marca `Degraded` e retorna `SpiError`.
    fn write_reg(&mut self, addr: u8, reg: u8, val: u8) -> Result<(), SensorError> {
        let opcode = write_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            let write_result = spi.blocking_write(&[opcode, reg, val]).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                SensorError::SpiError
            });
            self.cs.set_high();
            write_result?;
            Ok(())
        })
        .map_err(|_| {
            self.health = SensorHealth::Failed;
            SensorError::NotInitialized
        })?
    }

    /// Lê um registrador de um chip.
    /// Envia opcode + endereço + byte dummy (0xFF) para clock out dos dados.
    fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, SensorError> {
        let opcode = read_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            let mut buf = [opcode, reg, 0xFF];
            let transfer_result = spi.blocking_transfer_in_place(&mut buf).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                SensorError::SpiError
            });
            self.cs.set_high();
            transfer_result?;
            Ok(buf[2])
        })
        .map_err(|_| {
            self.health = SensorHealth::Failed;
            SensorError::NotInitialized
        })?
    }

    /// Lê GPIOA + GPIOB em uma única transação SPI (burst read).
    /// O MCP23S17 auto-incrementa o endereço do registrador, então ler de
    /// 0x12 (GPIOA) retorna GPIOA (0x12) e na sequência GPIOB (0x13).
    ///
    /// Retorno: u16 onde byte alto = GPIOB, byte baixo = GPIOA.
    fn read_chip_raw(&mut self, addr: u8) -> Result<u16, SensorError> {
        let opcode = read_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            // opcode + endereço GPIOA + 2 bytes dummy (MISO retorna GPIOA + GPIOB)
            let mut buf = [opcode, MCP23S17_GPIOA, 0x00, 0x00];
            let transfer_result = spi.blocking_transfer_in_place(&mut buf).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                SensorError::SpiError
            });
            self.cs.set_high();
            transfer_result?;
            // buf[2] = GPIOA, buf[3] = GPIOB
            // Leitura bem-sucedida: se health estava Degraded (erro SPI recuperado),
            // restaura para Healthy.
            self.health = SensorHealth::Healthy;
            Ok((buf[3] as u16) << 8 | buf[2] as u16)
        })
        .map_err(|_| {
            self.health = SensorHealth::Failed;
            SensorError::NotInitialized
        })?
    }

    /// Filtro de debounce: só atualiza `state` após N leituras idênticas.
    ///
    /// Se a leitura atual difere da anterior, reinicia o contador.
    /// Se é igual e o contador atinge `threshold`, atualiza `state`.
    /// Isso rejeita bouncing sem usar temporizadores — o período de amostragem
    /// é definido pelo ciclo do input_task (500μs).
    fn debounce_chip(chip: &mut ChipState, raw: u16, threshold: u8) {
        if raw == chip.raw_prev {
            chip.stable_cnt = chip.stable_cnt.saturating_add(1);
            if chip.stable_cnt >= threshold {
                chip.state = raw;
            }
        } else {
            chip.stable_cnt = 0;
            chip.raw_prev = raw;
        }
    }

    /// Altera o threshold de debounce em runtime (via ButtonRuntimeConfig).
    /// Mínimo 1 = sem debounce (instantâneo, mas sujeito a bouncing).
    pub fn set_debounce_threshold(&mut self, readings: u8) {
        self.debounce_threshold = readings.max(1);
    }

    /// Runtime health check: read back IOCON from both chips and verify
    /// against the expected value (0x0C).
    ///
    /// Se um chip não responder (MISO preso, ausente), IOCON lido difere
    /// de 0x0C e o driver marca `health = Failed` + incrementa error_count.
    ///
    /// Esta verificação resolve o risco de "MISO preso em 0x00 após o boot"
    /// — o readback de IOCON no init só cobre o momento da inicialização.
    fn runtime_health_check(&mut self) {
        let iocon0 = match self.read_reg(CHIP_ADDR_U1, MCP23S17_IOCON) {
            Ok(v) => v,
            Err(_) => {
                self.health = SensorHealth::Failed;
                return;
            }
        };
        let iocon1 = match self.read_reg(CHIP_ADDR_U2, MCP23S17_IOCON) {
            Ok(v) => v,
            Err(_) => {
                self.health = SensorHealth::Failed;
                return;
            }
        };

        if iocon0 != 0x0C || iocon1 != 0x0C {
            self.error_count = self.error_count.saturating_add(1);
            self.health = SensorHealth::Failed;
        }
    }
}

impl<'d> Sensor for Mcp23s<'d> {
    type Output = u32;

    /// Lê o estado combinado dos dois chips MCP23S17.
    ///
    /// Retorno: u32 onde os 16 bits superiores = U2, 16 bits inferiores = U1.
    /// Bit = 1 → liberado, Bit = 0 → pressionado (active low).
    ///
    /// Se a inicialização falhou, retorna `MCP23S17_BUTTONS_RELEASED` (0xFFFF_FFFF)
    /// sem tentar comunicação SPI — evita travamento.
    fn read(&mut self) -> Result<u32, SensorError> {
        if !self.available {
            return Ok(MCP23S17_BUTTONS_RELEASED);
        }

        // Periodic runtime health check: read back IOCON from both chips.
        // Detects MCP chips that died after boot (risk: MISO stuck at 0x00
        // would read as "all buttons pressed" without error flag).
        self.cycle_count = self.cycle_count.wrapping_add(1);
        if self.cycle_count.is_multiple_of(HEALTH_CHECK_INTERVAL) {
            self.runtime_health_check();
        }

        let raw0 = self.read_chip_raw(CHIP_ADDR_U1)?;
        let raw1 = self.read_chip_raw(CHIP_ADDR_U2)?;

        Self::debounce_chip(&mut self.chip0, raw0, self.debounce_threshold);
        Self::debounce_chip(&mut self.chip1, raw1, self.debounce_threshold);

        let merged = (self.chip1.state as u32) << 16 | self.chip0.state as u32;
        Ok(merged)
    }

    fn error_count(&self) -> u32 {
        self.error_count
    }

    fn health(&self) -> SensorHealth {
        self.health
    }
}
