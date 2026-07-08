//! Driver para sensor magnético MT6826S — encoder absoluto de 15 bits via SPI.
//!
//! # Hardware
//!
//! Três MT6826S no barramento SPI1 (mode 3: polarity=IdleHigh, phase=CaptureOnSecondTransition):
//! - X (roll): CS = PIN_13
//! - Y (pitch): CS = PIN_16
//! - Twist (yaw): CS = PIN_10
//!
//! Cada sensor responde ao comando Burst Angle Read (0x0A) com 3 bytes de dados
//! (ângulo 15-bit + status) + 1 byte CRC8. O ângulo de 15 bits ocupa os bits
//! 15..1 do frame (ANGLE_SHIFT=1), bit 0 é usado para paridade/status.
//!
//! # Proteções
//!
//! 1. Verificação de sensor ausente: MISO flutuante/preso retorna 0xFF ou
//!    0x00 em todos os bytes — detectado e reportado como `NotPresent`.
//! 2. CRC8: calculado sobre os 3 bytes de dados, comparado com o byte de CRC
//!    enviado pelo sensor. Protege contra ruído SPI.
//! 3. Magnet status: bit 5 do byte de status (buf[4]) indica campo magnético
//!    inválido ou subtensão no sensor — detectado como `MagnetError`.
//!
//! # Modos de falha não tratados
//!
//! - SPI sem timeout: se MISO travar, a transação nunca termina (ver spi_bus.rs).
//! - Uma posição real exatamente em 0 com status/CRC zerados é indistinguível
//!   de MISO preso em 0x00. O firmware prioriza fail-safe e reporta ausente.
//!
//! # no_std / heap
//!
//! Sem heap. block_for usa busy-wait (microssegundos) dentro da transação SPI.
//! CS é pino GPIO — toggle manual para controle de timing.

use super::{Sensor, SensorError, SensorHealth};
use crate::constants::{
    MT6826_ANGLE_MAX, MT6826_ANGLE_SHIFT, MT6826_CMD_READ_ANGLE, MT6826_CRC8_POLY,
    MT6826_CS_HOLD_US, MT6826_CS_SETUP_US, MT6826_MAGNET_OK_MASK,
};
use crate::spi_bus;
use embassy_rp::gpio::Output;
use embassy_time::{block_for, Duration};

#[derive(Debug)]
pub struct Mt6826<'d> {
    cs: Output<'d>,
    /// Total errors (CRC + magnet) since boot.
    error_count: u32,
    /// CRC8 errors only (separate from magnet). V1.23.
    crc_error_count: u32,
    /// Magnet/voltage errors only. V1.23.
    magnet_error_count: u32,
    /// Current health status — updated on each `read()`.
    /// `Failed` means bus-level error (SPI not initialized).
    /// `Degraded` means sensor-level error (CRC, magnet, not present).
    health: SensorHealth,
}

impl<'d> Mt6826<'d> {
    pub fn new(cs: Output<'d>) -> Self {
        Self {
            cs,
            error_count: 0,
            crc_error_count: 0,
            magnet_error_count: 0,
            health: SensorHealth::Healthy,
        }
    }

    /// Calcula CRC8 polinômio 0x07 (mesmo do datasheet MT6826S, §8.6).
    ///
    /// Algoritmo: CRC-8 com polinômio 0x07 (x^8 + x^2 + x + 1), valor inicial 0x00,
    /// sem reflexão, sem XOR final (conforme especificação do sensor).
    ///
    /// Nota: `wrapping_shl` em vez de `<<` porque debug builds panicam em overflow
    /// com shift que excede o tamanho do tipo.
    fn compute_crc8(data: &[u8]) -> u8 {
        let mut crc: u8 = 0;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    // wrapping_shl: debug builds panic on overflow with plain <<
                    crc = crc.wrapping_shl(1) ^ MT6826_CRC8_POLY;
                } else {
                    crc = crc.wrapping_shl(1);
                }
            }
        }
        crc
    }

    /// Verifica se o campo magnético é válido.
    /// MT6826_MAGNET_OK_MASK = 0x06 (bits 2 e 1 do status byte).
    /// Ambos os bits devem ser 0 para campo OK.
    fn check_magnet(status: u8) -> bool {
        (status & MT6826_MAGNET_OK_MASK) == 0x00
    }
}

impl<'d> Sensor for Mt6826<'d> {
    type Output = u16;

    /// Executa uma transação Burst Angle Read no sensor.
    ///
    /// Frame SPI (6 bytes):
    /// - byte 0: comando (C3..C0 = 0x0A) no nibble superior (0xA0)
    /// - byte 1: endereço de registrador (0x03 = Burst Angle Read)
    /// - bytes 2-3: ângulo (15 bits) deslocado à direita por ANGLE_SHIFT
    /// - byte 4: status (bits 0=paridade, 1-2=magnet, 3=overflow, 5=selftest)
    /// - byte 5: CRC8 sobre bytes 2..4
    ///
    /// Retorna: ângulo de 15 bits (0..32767) após validação CRC + magnet.
    fn read(&mut self) -> Result<u16, SensorError> {
        spi_bus::with_spi1(|spi| {
            self.cs.set_low();
            // Datasheet TL: CSN deve permanecer baixo por pelo menos 100 ns
            // antes do primeiro falling edge de SCK.
            block_for(Duration::from_micros(MT6826_CS_SETUP_US));

            // MT6826_CMD_READ_ANGLE << 4: comando 0x0A no nibble superior = 0xA0.
            // 0x03: registrador Burst Angle Read.
            // bytes 2-5: preenchidos pelo sensor durante transfer.
            let mut buf = [MT6826_CMD_READ_ANGLE << 4, 0x03, 0x00, 0x00, 0x00, 0x00];
            let transfer_result = spi
                .blocking_transfer_in_place(&mut buf)
                .map_err(|_| SensorError::SpiError);

            // Datasheet TH: mantenha CSN baixo por pelo menos 0,5 * TSCK
            // depois do ultimo rising edge. Em 1 MHz, use 1 us de margem.
            block_for(Duration::from_micros(MT6826_CS_HOLD_US));
            self.cs.set_high();
            transfer_result?;

            // Sensor ausente ou MISO preso pode produzir todos os bytes em
            // 0xFF ou 0x00. O frame 0x00 passaria no CRC8, então rejeite antes
            // da validação de CRC para não reportar eixo ausente como saudável.
            let all_ff = buf[2..=5].iter().all(|byte| *byte == 0xFF);
            let all_zero = buf[2..=5].iter().all(|byte| *byte == 0x00);
            if all_ff || all_zero {
                self.error_count = self.error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                return Err(SensorError::NotPresent);
            }

            // Verifica CRC8 sobre bytes 2..4 (ângulo + status).
            // buf[5] é o CRC enviado pelo sensor.
            let crc_expected = Self::compute_crc8(&buf[2..5]);
            if crc_expected != buf[5] {
                self.error_count = self.error_count.saturating_add(1);
                self.crc_error_count = self.crc_error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                return Err(SensorError::CrcError);
            }

            // Verifica campo magnético (bits 1-2 do status byte = 0x06).
            // Se o ímã está muito distante ou falta tensão, o sensor avisa.
            if !Self::check_magnet(buf[4]) {
                self.error_count = self.error_count.saturating_add(1);
                self.magnet_error_count = self.magnet_error_count.saturating_add(1);
                self.health = SensorHealth::Degraded;
                return Err(SensorError::MagnetError);
            }

            // Extrai ângulo de 15 bits: bytes 2-3 (16 bits) deslocado 1 bit.
            // ANGLE_SHIFT = 1 porque o bit 0 não faz parte do ângulo (datasheet §8.6.8).
            let raw: u16 = (buf[2] as u16) << 8 | (buf[3] as u16);
            let angle = raw >> MT6826_ANGLE_SHIFT;

            // Clamp defensivo: CRC passou, mas ângulo não pode exceder 32767.
            // Um bit espúrio pode corromper o frame mesmo com CRC válido
            // (colisão CRC — extremamente raro).
            // Leitura bem-sucedida — restaura health para Healthy se estava Degraded.
            self.health = SensorHealth::Healthy;
            Ok(angle.min(MT6826_ANGLE_MAX))
        })
        // Se `with_spi1` retornar NotInitialized, converte para SensorError.
        // Este é um erro de BARRAMENTO: afeta todos os sensores no SPI1.
        // Marca como `Failed` — não é recuperável sem reboot.
        .map_err(|_| {
            self.health = SensorHealth::Failed;
            SensorError::NotInitialized
        })?
    }

    fn error_count(&self) -> u32 {
        self.error_count
    }

    fn health(&self) -> SensorHealth {
        self.health
    }
}

impl<'d> Mt6826<'d> {
    /// Apenas erros CRC8 (V1.23 — separado de magnet error).
    pub fn crc_error_count(&self) -> u32 {
        self.crc_error_count
    }

    /// Apenas erros de campo magnético / subtensão (V1.23).
    pub fn magnet_error_count(&self) -> u32 {
        self.magnet_error_count
    }
}
