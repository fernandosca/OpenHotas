//! Acesso à flash QSPI do RP2350 para persistência de configuração.
//!
//! # Mapa de memória
//!
//! A flash externa QSPI (2 MB) é mapeada em 0x10000000 (XIP) no RP2350.
//! As operações de escrita/erase usam a periférica `blocking` da embassy_rp,
//! que requer um lock no `Mutex<Flash>` global.
//!
//! # Concorrência
//!
//! Leitura XIP e escrita/erase NÃO podem ocorrer simultaneamente no RP2350:
//! acessar memória XIP enquanto uma operação de flash (erase/write) está em
//! andamento congela o barramento QSPI. O `Mutex` garante exclusão mútua.
//!
//! # Power fail
//!
//! - `erase_sector` apaga 4 KB. Se a energia cair no meio, o setor fica
//!   parcialmente apagado (conteúdo = 0xFF em bytes apagados, indeterminado
//!   nos não apagados). O double-buffer em `stored_config_v2` protege: o slot
//!   ativo nunca é tocado, o target fica inválido e o boot usa o slot ativo.
//! - `write_flash` escreve após o erase. Se a energia cair no meio, os dados
//!   ficam parcialmente escritos. O CRC na leitura detecta e descarta o slot.
//!
//! # Gaps conhecidos
//!
//! - Nenhuma operação faz readback para verificar se o erase/write realmente
//!   persistiram. A HAL retorna erro se a operação falhar, mas não valida
//!   o conteúdo escrito.
//! - Não há proteção contra escrita em região de código do firmware.
//!   O caller (stored_config_v2) usa offsets no final da flash, longe do
//!   código — mas não há verificação em runtime.
//!
//! # no_std / heap
//!
//! Sem heap. Mutex blocking, adequado para uso no boot e em tasks.

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

// Instância global única do periférico de flash. Inicializada uma vez no boot.
pub static FLASH_INSTANCE: Mutex<CriticalSectionRawMutex, RefCell<Option<OpenHotasFlash>>> =
    Mutex::new(RefCell::new(None));

/// Inicializa o driver de flash.
/// Deve ser chamado uma vez no boot (em main.rs), antes de qualquer
/// operação de leitura/escrita/erase.
pub fn init(flash: OpenHotasFlash) {
    FLASH_INSTANCE.lock(|state| {
        *state.borrow_mut() = Some(flash);
    });
}

/// Executa `f` com acesso `&mut` ao periférico de flash.
/// Retorna `NotInitialized` se `init` não foi chamado.
fn with_flash<R>(
    f: impl FnOnce(&mut OpenHotasFlash) -> Result<R, FlashError>,
) -> Result<R, FlashError> {
    FLASH_INSTANCE.lock(|state| {
        let mut flash_ref = state.borrow_mut();
        let flash = flash_ref.as_mut().ok_or(FlashError::NotInitialized)?;
        f(flash)
    })
}

/// Valida que `offset + len` está dentro dos limites da flash e não
/// causa overflow aritmético.
fn validate_range(offset: u32, len: usize) -> Result<(), FlashError> {
    let end = offset
        .checked_add(len as u32)
        .ok_or(FlashError::OutOfBounds)?;

    if end > FLASH_SIZE {
        return Err(FlashError::OutOfBounds);
    }

    Ok(())
}

/// Lê `buf.len()` bytes da flash no offset especificado via XIP.
///
/// A leitura é feita por acesso XIP memory-mapped (0x10000000 + offset),
/// DENTRO do lock do Mutex, para evitar conflito com erase/write simultâneos.
///
/// # Safety do bloco XIP
///
/// O ponteiro `ptr` é construído a partir de `offset` validado por
/// `validate_range`, e a leitura é feita com `read_volatile` para evitar
/// que o compilador otimize (otimização ilegal para memory-mapped I/O).
/// O lock garante que nenhuma operação de flash (erase/write) está em
/// andamento durante a leitura — condição exigida pelo RP2350.
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

        // XIP base do RP2350 = 0x10000000.
        let ptr = (0x10000000u32 + offset) as *const u8;
        for (i, byte) in buf.iter_mut().enumerate() {
            // Safety: XIP memory-mapped read-only access a offset validado.
            // Invariante: lock held, offset validado, ponteiro não-nulo.
            // read_volatile evita que o compilador coalesça ou elimine leituras.
            *byte = unsafe { core::ptr::read_volatile(ptr.add(i)) };
        }
        Ok(())
    })
}

/// Apaga um setor (4 KB) da flash.
///
/// O offset DEVE ser alinhado a SECTOR_SIZE (4096) — exigência de hardware
/// do controlador QSPI do RP2350. Retorna `InvalidOffset` se não estiver.
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

/// Escreve dados na flash.
///
/// O offset DEVE ser alinhado a SECTOR_SIZE. O setor destino deve ter sido
/// apagado antes (a HAL não faz erase automático).
/// Retorna `InvalidOffset` se o offset não estiver alinhado.
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

// Re-export para conveniência dos callers (stored_config_v2).
pub use openhotas_filters::crc32;
