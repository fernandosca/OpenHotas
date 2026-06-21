//! StoredConfigV2 — Flash persistence for protocol DeviceConfig.
//!
//! V1.22 replaces the old manual layout (settings.rs) with postcard-serialized
//! protocol DeviceConfig. The flash layout:
//!
//! ```text
//! Offset  Size  Field
//! 0       4     MAGIC = "OCFG"
//! 4       1     STORAGE_VERSION = 2
//! 5       1     PROTOCOL_MAJOR
//! 6       1     PROTOCOL_MINOR
//! 7       2     PAYLOAD_LEN u16 big-endian
//! 9       N     PAYLOAD = postcard(DeviceConfig)
//! 9+N     4     CRC32 (covers bytes 4 through 9+N-1)
//! ```

use core::sync::atomic::Ordering;

use crate::constants::STORED_V2_OFFSET;
use crate::diagnostics::runtime_stats;
use crate::storage::flash;
use openhotas_protocol::config::DeviceConfig;
use openhotas_protocol::version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};

/// Magic number for V2 config: "OCFG" (OpenHotas ConFiG).
const MAGIC_V2: [u8; 4] = [b'O', b'C', b'F', b'G'];
const STORAGE_VERSION: u8 = 2;
/// Maximum payload size for postcard-serialized DeviceConfig.
const MAX_PAYLOAD: usize = 256;
/// Total buffer size: MAGIC(4)+VERSION(1)+MAJOR(1)+MINOR(1)+LEN(2)+MAX_PAYLOAD(256)+CRC(4)
const BUF_SIZE: usize = 4 + 1 + 1 + 1 + 2 + MAX_PAYLOAD + 4;

/// Load protocol DeviceConfig from flash, or return default.
pub fn load_config() -> DeviceConfig {
    let mut buf = [0u8; BUF_SIZE];
    if flash::read_flash(STORED_V2_OFFSET, &mut buf).is_err() {
        return DeviceConfig::default();
    }

    // Check magic
    if buf[0..4] != MAGIC_V2 {
        return DeviceConfig::default();
    }

    // Check storage version
    if buf[4] != STORAGE_VERSION {
        return DeviceConfig::default();
    }

    let proto_major = buf[5];
    let _proto_minor = buf[6];
    // If protocol major doesn't match, return default
    if proto_major != PROTOCOL_VERSION_MAJOR {
        return DeviceConfig::default();
    }

    let payload_len = u16::from_be_bytes([buf[7], buf[8]]) as usize;
    if payload_len > MAX_PAYLOAD {
        return DeviceConfig::default();
    }

    let payload_end = 9 + payload_len;
    let crc_offset = payload_end;

    // CRC32 covers storage_version (1) + proto_major (1) + proto_minor (1) + payload_len (2) + payload (N)
    let computed_crc = flash::crc32(&buf[4..crc_offset]);

    let mut stored_crc_bytes = [0u8; 4];
    stored_crc_bytes.copy_from_slice(&buf[crc_offset..crc_offset + 4]);
    let stored_crc = u32::from_le_bytes(stored_crc_bytes);

    if stored_crc != computed_crc {
        return DeviceConfig::default();
    }

    postcard::from_bytes::<DeviceConfig>(&buf[9..payload_end]).unwrap_or_default()
}

/// Save protocol DeviceConfig to flash.
///
/// Erases sector, writes header + postcard payload + CRC32.
pub fn save_config(config: &DeviceConfig) -> Result<(), flash::FlashError> {
    let mut buf = [0u8; BUF_SIZE];

    // Magic
    buf[0..4].copy_from_slice(&MAGIC_V2);

    // Header
    buf[4] = STORAGE_VERSION;
    buf[5] = PROTOCOL_VERSION_MAJOR;
    buf[6] = PROTOCOL_VERSION_MINOR;

    // Serialize config to payload area
    let payload_buf = &mut [0u8; MAX_PAYLOAD];
    let payload =
        postcard::to_slice(config, payload_buf).map_err(|_| flash::FlashError::WriteError)?;

    // Verify payload fits in buffer (defensive check)
    if payload.len() > MAX_PAYLOAD {
        return Err(flash::FlashError::WriteError);
    }

    let payload_len = payload.len();
    buf[7] = (payload_len as u16 >> 8) as u8;
    buf[8] = payload_len as u8;
    buf[9..9 + payload_len].copy_from_slice(payload);

    let payload_end = 9 + payload_len;
    // CRC32 covers from buf[4] to payload_end
    let crc = flash::crc32(&buf[4..payload_end]);
    buf[payload_end..payload_end + 4].copy_from_slice(&crc.to_le_bytes());

    let total_len = payload_end + 4;

    if let Err(e) = flash::erase_sector(STORED_V2_OFFSET) {
        runtime_stats::FLASH_ERRORS.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }
    // V1.24: write only actual data bytes. Sector already erased →
    // unwritten bytes remain 0xFF. No 4KB stack buffer needed.
    if let Err(e) = flash::write_flash(STORED_V2_OFFSET, &buf[..total_len]) {
        runtime_stats::FLASH_ERRORS.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }

    Ok(())
}
