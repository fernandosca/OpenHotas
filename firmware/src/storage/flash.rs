use crate::constants::{FLASH_SIZE, SECTOR_SIZE};
use core::cell::RefCell;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;

pub type OpenHotasFlash = Flash<'static, FLASH, Blocking, { 2 * 1024 * 1024 }>;

#[derive(Debug)]
pub enum FlashError {
    WriteError,
    EraseError,
    InvalidOffset,
    OutOfBounds,
    NotInitialized,
}

pub static FLASH_INSTANCE: Mutex<CriticalSectionRawMutex, RefCell<Option<OpenHotasFlash>>> =
    Mutex::new(RefCell::new(None));

pub fn init(flash: OpenHotasFlash) {
    FLASH_INSTANCE.lock(|state| {
        *state.borrow_mut() = Some(flash);
    });
}

fn with_flash<R>(
    f: impl FnOnce(&mut OpenHotasFlash) -> Result<R, FlashError>,
) -> Result<R, FlashError> {
    FLASH_INSTANCE.lock(|state| {
        let mut flash_ref = state.borrow_mut();
        let flash = flash_ref.as_mut().ok_or(FlashError::NotInitialized)?;
        f(flash)
    })
}

fn validate_range(offset: u32, len: usize) -> Result<(), FlashError> {
    let end = offset
        .checked_add(len as u32)
        .ok_or(FlashError::OutOfBounds)?;

    if end > FLASH_SIZE {
        return Err(FlashError::OutOfBounds);
    }

    Ok(())
}

pub fn read_flash(offset: u32, buf: &mut [u8]) -> Result<(), FlashError> {
    validate_range(offset, buf.len())?;

    // Leitura XIP mantida DENTRO do lock.
    //
    // No RP2350, qualquer acesso XIP durante um `blocking_erase`/`blocking_write`
    // congela o barramento. Segurar o lock aqui garante que a leitura não
    // concorra com uma escrita/erase — corrigindo o TOCTOU da versão anterior,
    // onde a verificação `is_none()` liberava o mutex antes da leitura XIP.
    // Hoje só é usado no boot, mas assim a API fica correta sob uso concorrente.
    FLASH_INSTANCE.lock(|state| {
        if state.borrow().is_none() {
            return Err(FlashError::NotInitialized);
        }

        let ptr = (0x10000000u32 + offset) as *const u8;
        for (i, byte) in buf.iter_mut().enumerate() {
            // Safety: XIP memory-mapped read-only access a offset validado.
            *byte = unsafe { core::ptr::read_volatile(ptr.add(i)) };
        }
        Ok(())
    })
}

pub fn erase_sector(offset: u32) -> Result<(), FlashError> {
    if !offset.is_multiple_of(SECTOR_SIZE) {
        return Err(FlashError::InvalidOffset);
    }

    validate_range(offset, SECTOR_SIZE as usize)?;

    with_flash(|flash| {
        flash
            .blocking_erase(offset, offset + SECTOR_SIZE)
            .map_err(|_| FlashError::EraseError)
    })
}

pub fn write_flash(offset: u32, data: &[u8]) -> Result<(), FlashError> {
    validate_range(offset, data.len())?;

    if !offset.is_multiple_of(SECTOR_SIZE) {
        return Err(FlashError::InvalidOffset);
    }

    with_flash(|flash| {
        flash
            .blocking_write(offset, data)
            .map_err(|_| FlashError::WriteError)
    })
}

pub use openhotas_filters::crc32;
