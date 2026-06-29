use super::{Sensor, SensorError};
use crate::constants::{
    MT6826_ANGLE_MAX, MT6826_ANGLE_SHIFT, MT6826_CRC8_POLY, MT6826_CS_RELEASE_US,
    MT6826_MAGNET_OK_MASK,
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
}

impl<'d> Mt6826<'d> {
    pub fn new(cs: Output<'d>) -> Self {
        Self {
            cs,
            error_count: 0,
            crc_error_count: 0,
            magnet_error_count: 0,
        }
    }

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
            let transfer_result = spi
                .blocking_transfer_in_place(&mut buf)
                .map_err(|_| SensorError::SpiError);

            self.cs.set_high();
            // Os tres sensores compartilham MISO. Aguarde este dispositivo
            // liberar a linha antes que o proximo CS possa ser selecionado.
            block_for(Duration::from_micros(MT6826_CS_RELEASE_US));
            transfer_result?;

            // A disconnected bus can float low and produce [0, 0, 0, 0].
            // That pattern has a mathematically valid CRC, but it is not a
            // trustworthy sensor response. Reject it before CRC validation.
            if buf[2..=5].iter().all(|byte| *byte == 0) {
                self.error_count = self.error_count.saturating_add(1);
                return Err(SensorError::NotPresent);
            }

            let crc_expected = Self::compute_crc8(&buf[2..5]);
            if crc_expected != buf[5] {
                self.error_count = self.error_count.saturating_add(1);
                self.crc_error_count = self.crc_error_count.saturating_add(1);
                return Err(SensorError::CrcError);
            }

            if !Self::check_magnet(buf[4]) {
                self.error_count = self.error_count.saturating_add(1);
                self.magnet_error_count = self.magnet_error_count.saturating_add(1);
                return Err(SensorError::MagnetError);
            }

            let raw: u16 = (buf[2] as u16) << 8 | (buf[3] as u16);
            let angle = raw >> MT6826_ANGLE_SHIFT;

            Ok(angle.min(MT6826_ANGLE_MAX))
        })
        .map_err(|_| SensorError::NotInitialized)?
    }

    fn error_count(&self) -> u32 {
        self.error_count
    }
}

impl<'d> Mt6826<'d> {
    /// CRC8 errors only (V1.23 — separate from magnet).
    pub fn crc_error_count(&self) -> u32 {
        self.crc_error_count
    }

    /// Magnet/voltage errors only (V1.23).
    pub fn magnet_error_count(&self) -> u32 {
        self.magnet_error_count
    }
}
