use super::data::CalibrationData;
use crate::constants::{AXIS_COUNT, CALIB_OFFSET, MAGIC_CAL};
use crate::storage::flash;

const HEADER_SIZE: usize = 4;
const DATA_PER_AXIS: usize = 6;
const CRC_SIZE: usize = 4;
const TOTAL_SIZE: usize = HEADER_SIZE + (DATA_PER_AXIS * AXIS_COUNT) + CRC_SIZE;

#[allow(dead_code)]
pub fn save(axes: &[CalibrationData; 3]) -> Result<(), flash::FlashError> {
    let mut buf = [0u8; TOTAL_SIZE];

    buf[0..4].copy_from_slice(&MAGIC_CAL.to_be_bytes());

    for (i, axis) in axes.iter().enumerate() {
        let off = HEADER_SIZE + (i * DATA_PER_AXIS);
        buf[off..off + 2].copy_from_slice(&axis.center.to_le_bytes());
        buf[off + 2..off + 4].copy_from_slice(&axis.min.to_le_bytes());
        buf[off + 4..off + 6].copy_from_slice(&axis.max.to_le_bytes());
    }

    let crc = flash::crc32(&buf[0..TOTAL_SIZE - CRC_SIZE]);
    let crc_off = TOTAL_SIZE - CRC_SIZE;
    buf[crc_off..crc_off + 4].copy_from_slice(&crc.to_le_bytes());

    flash::erase_sector(CALIB_OFFSET)?;
    flash::write_flash(CALIB_OFFSET, &buf)?;
    Ok(())
}

pub fn load() -> [CalibrationData; 3] {
    let mut buf = [0u8; TOTAL_SIZE];
    if flash::read_flash(CALIB_OFFSET, &mut buf).is_err() {
        return [CalibrationData::default(); 3];
    }

    let mut magic_bytes = [0u8; 4];
    magic_bytes.copy_from_slice(&buf[0..4]);
    let magic = u32::from_be_bytes(magic_bytes);
    if magic != MAGIC_CAL {
        return [CalibrationData::default(); 3];
    }

    let stored_crc = {
        let crc_off = TOTAL_SIZE - CRC_SIZE;
        let mut crc_bytes = [0u8; 4];
        crc_bytes.copy_from_slice(&buf[crc_off..crc_off + 4]);
        u32::from_le_bytes(crc_bytes)
    };

    let computed_crc = flash::crc32(&buf[0..TOTAL_SIZE - CRC_SIZE]);
    if stored_crc != computed_crc {
        return [CalibrationData::default(); 3];
    }

    let mut axes = [CalibrationData::default(); 3];
    for (i, axis) in axes.iter_mut().enumerate().take(AXIS_COUNT) {
        let off = HEADER_SIZE + (i * DATA_PER_AXIS);
        axis.center = u16::from_le_bytes([buf[off], buf[off + 1]]);
        axis.min = u16::from_le_bytes([buf[off + 2], buf[off + 3]]);
        axis.max = u16::from_le_bytes([buf[off + 4], buf[off + 5]]);
    }
    axes
}
