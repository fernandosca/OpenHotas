use super::{Sensor, SensorError};
use crate::constants::{
    MCP23S17_DEBOUNCE_COUNT, MCP23S17_GPIOA, MCP23S17_GPPUA, MCP23S17_GPPUB, MCP23S17_IOCON,
    MCP23S17_IODIRA, MCP23S17_IODIRB,
};
use crate::spi_bus;
use embassy_rp::gpio::Output;

const CHIP_ADDR_U1: u8 = 0x00;
const CHIP_ADDR_U2: u8 = 0x01;
const MCP23S17_BUTTONS_RELEASED: u32 = 0xFFFF_FFFF;

fn write_opcode(addr: u8) -> u8 {
    0x40 | (addr << 1)
}

fn read_opcode(addr: u8) -> u8 {
    0x41 | (addr << 1)
}

#[derive(Debug)]
struct ChipState {
    state: u16,
    raw_prev: u16,
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
    /// Debounce threshold (consecutive stable readings required).
    /// Default: MCP23S17_DEBOUNCE_COUNT. Configurable via ButtonRuntimeConfig.
    debounce_threshold: u8,
    available: bool,
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
        }
    }

    pub fn init(&mut self) -> Result<(), SensorError> {
        let result = self.init_chips();
        self.available = result.is_ok();
        result
    }

    fn init_chips(&mut self) -> Result<(), SensorError> {
        for addr in [CHIP_ADDR_U1, CHIP_ADDR_U2] {
            self.write_reg(addr, MCP23S17_IOCON, 0x0C)?;
            self.write_reg(addr, MCP23S17_IODIRA, 0xFF)?;
            self.write_reg(addr, MCP23S17_IODIRB, 0xFF)?;
            self.write_reg(addr, MCP23S17_GPPUA, 0xFF)?;
            self.write_reg(addr, MCP23S17_GPPUB, 0xFF)?;
        }
        Ok(())
    }

    fn write_reg(&mut self, addr: u8, reg: u8, val: u8) -> Result<(), SensorError> {
        let opcode = write_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            let write_result = spi.blocking_write(&[opcode, reg, val]).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                SensorError::SpiError
            });
            self.cs.set_high();
            write_result?;
            Ok(())
        })
        .map_err(|_| SensorError::NotInitialized)?
    }

    /// Read GPIOA + GPIOB in a single SPI transaction (burst read).
    /// MCP23S17 auto-increments register address, so reading from 0x12
    /// returns GPIOA (0x12) then GPIOB (0x13) sequentially.
    fn read_chip_raw(&mut self, addr: u8) -> Result<u16, SensorError> {
        let opcode = read_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            // opcode + GPIOA address + 2 dummy bytes for GPIOA + GPIOB
            let mut buf = [opcode, MCP23S17_GPIOA, 0x00, 0x00];
            let transfer_result = spi.blocking_transfer_in_place(&mut buf).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                SensorError::SpiError
            });
            self.cs.set_high();
            transfer_result?;
            // buf[2] = GPIOA, buf[3] = GPIOB
            Ok((buf[3] as u16) << 8 | buf[2] as u16)
        })
        .map_err(|_| SensorError::NotInitialized)?
    }

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

    /// Update debounce threshold at runtime (from ButtonRuntimeConfig).
    /// Clamped to minimum 1 (no debounce → instant response).
    pub fn set_debounce_threshold(&mut self, readings: u8) {
        self.debounce_threshold = readings.max(1);
    }
}

impl<'d> Sensor for Mcp23s<'d> {
    type Output = u32;

    fn read(&mut self) -> Result<u32, SensorError> {
        if !self.available {
            return Ok(MCP23S17_BUTTONS_RELEASED);
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
}
