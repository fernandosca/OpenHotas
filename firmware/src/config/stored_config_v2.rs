//! StoredConfigV2 — Flash persistence with double-buffer for power-fail safety.
//!
//! Two slots (A and B) alternate writes. Each slot has a generation counter
//! that increments on every save. On boot, the valid slot with the highest
//! generation is used. This ensures no data loss even if power fails during
//! erase or write.
//!
//! Flash layout per slot:
//!
//! ```text
//! Offset  Size  Field
//! 0       4     MAGIC = "OCFG"
//! 4       4     GENERATION (u32 LE, increments on each save)
//! 8       1     STORAGE_VERSION = 2
//! 9       1     PROTOCOL_MAJOR
//! 10      1     PROTOCOL_MINOR
//! 11      2     PAYLOAD_LEN u16 big-endian
//! 13      N     PAYLOAD = postcard(DeviceConfig)
//! 13+N    4     CRC32 (covers bytes 4 through 13+N-1)
//! ```

use core::sync::atomic::Ordering;

use crate::constants::{STORED_V2_SLOT_A, STORED_V2_SLOT_B};
use crate::diagnostics::runtime_stats;
use crate::storage::flash;
use openhotas_protocol::config::DeviceConfig;
use openhotas_protocol::version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};

/// Magic number for V2 config: "OCFG" (OpenHotas ConFiG).
const MAGIC_V2: [u8; 4] = [b'O', b'C', b'F', b'G'];
const STORAGE_VERSION: u8 = 2;
/// Maximum payload size for postcard-serialized DeviceConfig.
const MAX_PAYLOAD: usize = 256;
/// Header size: MAGIC(4) + GENERATION(4) + VERSION(1) + MAJOR(1) + MINOR(1) + LEN(2) = 13
const HEADER_SIZE: usize = 13;
/// CRC size: 4 bytes
const CRC_SIZE: usize = 4;
/// Total buffer size: HEADER(13) + MAX_PAYLOAD(256) + CRC(4) = 273
const BUF_SIZE: usize = HEADER_SIZE + MAX_PAYLOAD + CRC_SIZE;

/// Slot state after validation.
struct SlotData {
    generation: u32,
    config: DeviceConfig,
}

/// Read and validate a single slot. Returns `Some(SlotData)` if valid.
fn read_slot(offset: u32) -> Option<SlotData> {
    let mut buf = [0u8; BUF_SIZE];
    if flash::read_flash(offset, &mut buf).is_err() {
        return None;
    }

    // Check magic
    if buf[0..4] != MAGIC_V2 {
        return None;
    }

    // Read generation (u32 LE at offset 4)
    let generation = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

    // Check storage version
    if buf[8] != STORAGE_VERSION {
        return None;
    }

    let proto_major = buf[9];
    let _proto_minor = buf[10];
    // If protocol major doesn't match, return None
    if proto_major != PROTOCOL_VERSION_MAJOR {
        return None;
    }

    let payload_len = u16::from_be_bytes([buf[11], buf[12]]) as usize;
    if payload_len > MAX_PAYLOAD {
        return None;
    }

    let payload_end = HEADER_SIZE + payload_len;
    let crc_offset = payload_end;

    // CRC32 covers generation(4) + version(1) + major(1) + minor(1) + payload_len(2) + payload(N)
    let computed_crc = flash::crc32(&buf[4..crc_offset]);

    let mut stored_crc_bytes = [0u8; 4];
    stored_crc_bytes.copy_from_slice(&buf[crc_offset..crc_offset + CRC_SIZE]);
    let stored_crc = u32::from_le_bytes(stored_crc_bytes);

    if stored_crc != computed_crc {
        return None;
    }

    let config =
        postcard::from_bytes::<DeviceConfig>(&buf[HEADER_SIZE..payload_end]).unwrap_or_default();

    Some(SlotData { generation, config })
}

/// Load protocol DeviceConfig from flash, or return default.
///
/// Reads both slots, validates CRC of each, and uses the one with the
/// highest generation among valid ones.
pub fn load_config() -> DeviceConfig {
    let slot_a = read_slot(STORED_V2_SLOT_A);
    let slot_b = read_slot(STORED_V2_SLOT_B);

    match (slot_a, slot_b) {
        (Some(a), Some(b)) => {
            // Both valid — use highest generation
            if a.generation >= b.generation {
                a.config
            } else {
                b.config
            }
        }
        (Some(a), None) => a.config,
        (None, Some(b)) => b.config,
        (None, None) => DeviceConfig::default(),
    }
}

/// Save protocol DeviceConfig to flash using double-buffer.
///
/// Writes to the inactive slot (not the one with highest generation),
/// with generation = current + 1. This ensures no window where both
/// slots are invalid.
pub fn save_config(config: &DeviceConfig) -> Result<(), flash::FlashError> {
    // Determine which slot is active (highest generation)
    let slot_a = read_slot(STORED_V2_SLOT_A);
    let slot_b = read_slot(STORED_V2_SLOT_B);

    let (active_offset, next_generation) = match (&slot_a, &slot_b) {
        (Some(a), Some(b)) => {
            if a.generation >= b.generation {
                (STORED_V2_SLOT_A, a.generation.wrapping_add(1))
            } else {
                (STORED_V2_SLOT_B, b.generation.wrapping_add(1))
            }
        }
        (Some(a), None) => (STORED_V2_SLOT_A, a.generation.wrapping_add(1)),
        (None, Some(b)) => (STORED_V2_SLOT_B, b.generation.wrapping_add(1)),
        (None, None) => (STORED_V2_SLOT_A, 1),
    };

    // Write to the inactive slot
    let target_offset = if active_offset == STORED_V2_SLOT_A {
        STORED_V2_SLOT_B
    } else {
        STORED_V2_SLOT_A
    };

    let mut buf = [0u8; BUF_SIZE];

    // Magic
    buf[0..4].copy_from_slice(&MAGIC_V2);

    // Generation (u32 LE)
    buf[4..8].copy_from_slice(&next_generation.to_le_bytes());

    // Header
    buf[8] = STORAGE_VERSION;
    buf[9] = PROTOCOL_VERSION_MAJOR;
    buf[10] = PROTOCOL_VERSION_MINOR;

    // Serialize config to payload area
    let payload_buf = &mut [0u8; MAX_PAYLOAD];
    let payload =
        postcard::to_slice(config, payload_buf).map_err(|_| flash::FlashError::WriteError)?;

    // Verify payload fits in buffer (defensive check)
    if payload.len() > MAX_PAYLOAD {
        return Err(flash::FlashError::WriteError);
    }

    let payload_len = payload.len();
    buf[11] = (payload_len as u16 >> 8) as u8;
    buf[12] = payload_len as u8;
    buf[HEADER_SIZE..HEADER_SIZE + payload_len].copy_from_slice(payload);

    let payload_end = HEADER_SIZE + payload_len;
    // CRC32 covers from buf[4] (generation) to payload_end
    let crc = flash::crc32(&buf[4..payload_end]);
    buf[payload_end..payload_end + CRC_SIZE].copy_from_slice(&crc.to_le_bytes());

    let total_len = payload_end + CRC_SIZE;

    // Erase the target slot first
    if let Err(e) = flash::erase_sector(target_offset) {
        runtime_stats::FLASH_ERRORS.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }

    // Write to the target slot
    if let Err(e) = flash::write_flash(target_offset, &buf[..total_len]) {
        runtime_stats::FLASH_ERRORS.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }

    Ok(())
}
