use super::{Sensor, SensorError};
use crate::constants::{
    MT6826_ANGLE_MAX, MT6826_ANGLE_SHIFT, MT6826_CRC8_POLY, MT6826_MAGNET_OK_MASK,
};
use crate::spi_bus;
use embassy_rp::gpio::Output;

#[derive(Debug)]
pub struct Mt6826<'d> {
    cs: Output<'d>,
    error_count: u32,
    last_healthy: bool,
}

impl<'d> Mt6826<'d> {
    pub fn new(cs: Output<'d>) -> Self {
        Self {
            cs,
            error_count: 0,
            last_healthy: false,
        }
    }

    fn compute_crc8(data: &[u8]) -> u8 {
        let mut crc: u8 = 0;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    crc = (crc << 1) ^ MT6826_CRC8_POLY;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    fn check_magnet(status: u8) -> bool {
        (status & MT6826_MAGNET_OK_MASK) == 0x00
    }
}

impl<'d> Sensor for Mt6826<'d> {
    type Output = u16;

    fn read(&mut self) -> Result<u16, SensorError> {
        spi_bus::with_spi1(|spi| {
            self.cs.set_low();

            let mut buf = [0xA0u8, 0x03, 0x00, 0x00, 0x00, 0x00];
            spi.blocking_transfer_in_place(&mut buf)
                .map_err(|_| SensorError::SpiError)?;

            self.cs.set_high();

            let crc_expected = Self::compute_crc8(&buf[2..5]);
            if crc_expected != buf[5] {
                self.error_count = self.error_count.saturating_add(1);
                self.last_healthy = false;
                return Err(SensorError::CrcError);
            }

            if !Self::check_magnet(buf[4]) {
                self.last_healthy = false;
                return Err(SensorError::MagnetError);
            }

            let raw: u16 = (buf[2] as u16) << 8 | buf[3] as u16;
            let angle = raw >> MT6826_ANGLE_SHIFT;

            self.last_healthy = true;
            Ok(angle.min(MT6826_ANGLE_MAX))
        })
    }

    fn is_healthy(&self) -> bool {
        self.last_healthy
    }

    fn error_count(&self) -> u32 {
        self.error_count
    }
}
