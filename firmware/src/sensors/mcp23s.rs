use super::{Sensor, SensorError};
use crate::constants::{
    MCP23S17_DEBOUNCE_COUNT, MCP23S17_GPIOA, MCP23S17_GPIOB, MCP23S17_GPPUA, MCP23S17_GPPUB,
    MCP23S17_IOCON, MCP23S17_IODIRA, MCP23S17_IODIRB,
};
use crate::spi_bus;
use embassy_rp::gpio::Output;

const CHIP_ADDR_U1: u8 = 0x00;
const CHIP_ADDR_U2: u8 = 0x01;

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
}

impl<'d> Mcp23s<'d> {
    pub fn new(cs: Output<'d>) -> Self {
        Self {
            cs,
            chip0: ChipState::new(),
            chip1: ChipState::new(),
            error_count: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), SensorError> {
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
            spi.blocking_write(&[opcode, reg, val]).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                SensorError::SpiError
            })?;
            self.cs.set_high();
            Ok(())
        })
    }

    fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, SensorError> {
        let opcode = read_opcode(addr);
        spi_bus::with_spi0(|spi| {
            self.cs.set_low();
            let mut buf = [opcode, reg, 0x00];
            spi.blocking_transfer_in_place(&mut buf).map_err(|_| {
                self.error_count = self.error_count.saturating_add(1);
                SensorError::SpiError
            })?;
            self.cs.set_high();
            Ok(buf[2])
        })
    }

    fn read_chip_raw(&mut self, addr: u8) -> Result<u16, SensorError> {
        let a = self.read_reg(addr, MCP23S17_GPIOA)?;
        let b = self.read_reg(addr, MCP23S17_GPIOB)?;
        Ok((b as u16) << 8 | a as u16)
    }

    fn debounce_chip(chip: &mut ChipState, raw: u16) {
        if raw == chip.raw_prev {
            chip.stable_cnt = chip.stable_cnt.saturating_add(1);
            if chip.stable_cnt >= MCP23S17_DEBOUNCE_COUNT {
                chip.state = raw;
            }
        } else {
            chip.stable_cnt = 0;
            chip.raw_prev = raw;
        }
    }
}

impl<'d> Sensor for Mcp23s<'d> {
    type Output = u32;

    fn read(&mut self) -> Result<u32, SensorError> {
        let raw0 = self.read_chip_raw(CHIP_ADDR_U1)?;
        let raw1 = self.read_chip_raw(CHIP_ADDR_U2)?;

        Self::debounce_chip(&mut self.chip0, raw0);
        Self::debounce_chip(&mut self.chip1, raw1);

        let merged = (self.chip1.state as u32) << 16 | self.chip0.state as u32;
        Ok(merged)
    }

    fn is_healthy(&self) -> bool {
        self.error_count == 0
    }

    fn error_count(&self) -> u32 {
        self.error_count
    }
}
