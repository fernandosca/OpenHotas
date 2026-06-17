use crate::axis::AxisConfig;
use crate::constants::{CONFIG_OFFSET, CONFIG_VERSION, MAGIC_DEVICE};
use crate::storage::flash;

const HEADER_SIZE: usize = 4 + 1 + 1;
const AXES_SIZE: usize = 6 * 4;
const CRC_SIZE: usize = 4;
const TOTAL_SIZE: usize = HEADER_SIZE + AXES_SIZE + CRC_SIZE;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeviceConfig {
    pub magic: u32,
    pub version: u8,
    pub active_profile: u8,
    pub axes: [AxisConfig; 3],
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            magic: MAGIC_DEVICE,
            version: CONFIG_VERSION,
            active_profile: 0,
            axes: {
                let mut axes = [AxisConfig::default(); 3];
                axes[2].reset_ema_on_dz = true;
                axes
            },
        }
    }
}

impl DeviceConfig {
    pub fn load() -> Self {
        let mut buf = [0u8; TOTAL_SIZE];
        if flash::read_flash(CONFIG_OFFSET, &mut buf).is_err() {
            return Self::default();
        }

        let magic = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if magic != MAGIC_DEVICE {
            return Self::default();
        }

        let version = buf[4];
        if version != CONFIG_VERSION {
            return Self::default();
        }

        let stored_crc = {
            let crc_off = TOTAL_SIZE - CRC_SIZE;
            u32::from_le_bytes([
                buf[crc_off],
                buf[crc_off + 1],
                buf[crc_off + 2],
                buf[crc_off + 3],
            ])
        };

        let computed_crc = flash::crc32(&buf[0..TOTAL_SIZE - CRC_SIZE]);
        if stored_crc != computed_crc {
            return Self::default();
        }

        let active_profile = buf[5];

        let axes: [AxisConfig; 3] = {
            let mut axes = [AxisConfig::default(); 3];
            let base = HEADER_SIZE;
            for (i, axis) in axes.iter_mut().enumerate() {
                let off = base + (i * 24);
                axis.ema_alpha =
                    f32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
                axis.deadzone =
                    f32::from_le_bytes([buf[off + 4], buf[off + 5], buf[off + 6], buf[off + 7]]);
                axis.max_jump =
                    f32::from_le_bytes([buf[off + 8], buf[off + 9], buf[off + 10], buf[off + 11]]);
                axis.expo = f32::from_le_bytes([
                    buf[off + 12],
                    buf[off + 13],
                    buf[off + 14],
                    buf[off + 15],
                ]);
                axis.inverted = buf[off + 16] != 0;
                axis.reset_ema_on_dz = buf[off + 17] != 0;
            }
            axes
        };

        Self {
            magic,
            version,
            active_profile,
            axes,
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), flash::FlashError> {
        let mut buf = [0u8; TOTAL_SIZE];

        buf[0..4].copy_from_slice(&self.magic.to_be_bytes());
        buf[4] = self.version;
        buf[5] = self.active_profile;

        let base = HEADER_SIZE;
        for (i, axis) in self.axes.iter().enumerate() {
            let off = base + (i * 24);
            buf[off..off + 4].copy_from_slice(&axis.ema_alpha.to_le_bytes());
            buf[off + 4..off + 8].copy_from_slice(&axis.deadzone.to_le_bytes());
            buf[off + 8..off + 12].copy_from_slice(&axis.max_jump.to_le_bytes());
            buf[off + 12..off + 16].copy_from_slice(&axis.expo.to_le_bytes());
            buf[off + 16] = axis.inverted as u8;
            buf[off + 17] = axis.reset_ema_on_dz as u8;
        }

        let crc = flash::crc32(&buf[0..TOTAL_SIZE - CRC_SIZE]);
        let crc_off = TOTAL_SIZE - CRC_SIZE;
        buf[crc_off..crc_off + 4].copy_from_slice(&crc.to_le_bytes());

        flash::erase_sector(CONFIG_OFFSET)?;
        flash::write_flash(CONFIG_OFFSET, &buf)?;
        Ok(())
    }
}
