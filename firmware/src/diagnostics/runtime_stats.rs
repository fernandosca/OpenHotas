//! Estatísticas de runtime compartilhadas entre tasks via atomics.
//!
//! Valores atualizados por `input_task` e `hid_task`; lidos pelo
//! protocolo CDC (GetRuntimeStats, GetErrorCounters) e pela
//! `diagnostic_task` (log periódico).
//!
//! Todos os atomics usam `Ordering::Relaxed` — consistência sequencial
//! não é necessária porque os valores são amostrais (diagnóstico).
//! Uma leitura ver uma atualização parcial é aceitável.

use crate::constants::MAX_INPUT_CYCLE_US;
use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU8, Ordering};

// ── HID counters ───────────────────────────────────────────────────
/// Total de reports HID enviados com sucesso.
pub static REPORTS_SENT: AtomicU32 = AtomicU32::new(0);
/// Total de falhas de envio HID.
pub static SEND_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_CYCLES: AtomicU32 = AtomicU32::new(0);
pub static LAST_CYCLE_US: AtomicU32 = AtomicU32::new(0);
pub static MAX_CYCLE_US: AtomicU32 = AtomicU32::new(0);

// ── Valores crus dos sensores (para diagnóstico CDC) ───────────────
/// Último raw lido do sensor X (MT6826S).
pub static RAW_AXIS_X: AtomicU32 = AtomicU32::new(0);
pub static RAW_AXIS_Y: AtomicU32 = AtomicU32::new(0);
pub static RAW_AXIS_TWIST: AtomicU32 = AtomicU32::new(0);

// ── Valores processados (pós-pipeline) ─────────────────────────────
/// V1.25: AtomicI32 — valores signed (-32767..+32767).
pub static PROC_AXIS_X: AtomicI32 = AtomicI32::new(0);
pub static PROC_AXIS_Y: AtomicI32 = AtomicI32::new(0);
pub static PROC_AXIS_TWIST: AtomicI32 = AtomicI32::new(0);

/// Estado atual dos botões (32 bits).
pub static BUTTON_MASK: AtomicU32 = AtomicU32::new(0);

/// Bitmask de saúde dos sensores: bit0=X unhealthy, bit1=Y, bit2=Twist.
pub static SENSOR_UNHEALTHY: AtomicU8 = AtomicU8::new(0);

// ── Contadores de erro ─────────────────────────────────────────────
/// V1.23: erros de CRC16-CCITT em frames CDC.
pub static PROTOCOL_CRC_ERRORS: AtomicU32 = AtomicU32::new(0);
/// V1.23: erros de CRC8 nos sensores MT6826S.
pub static SENSOR_CRC_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static MAGNET_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static FLASH_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static BUTTON_ERRORS: AtomicU32 = AtomicU32::new(0);
/// 1 se o barramento de botões está em modo degradado (init falhou).
pub static BUTTONS_DEGRADED: AtomicU8 = AtomicU8::new(0);

// ── Contadores de erro por sensor (V1.22) ──────────────────────────
/// Delta acumulado de `SpiError` + `NotPresent` para cada sensor MT6826S.
/// Atualizado em input_task via `track_delta!`.
pub static SENSOR_X_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_Y_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_TWIST_ERRORS: AtomicU32 = AtomicU32::new(0);

pub fn record_report_sent() {
    REPORTS_SENT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_send_error() {
    SEND_ERRORS.fetch_add(1, Ordering::Relaxed);
}

/// Registra a duração de um ciclo do input_task.
/// Atualiza LAST, MAX (com compare_exchange loop para atomicidade) e
/// emite warning se exceder MAX_INPUT_CYCLE_US.
pub fn record_cycle(us: u32) {
    SENSOR_CYCLES.fetch_add(1, Ordering::Relaxed);
    LAST_CYCLE_US.store(us, Ordering::Relaxed);

    // MAX_CYCLE_US: atualização lock-free via compare_exchange loop.
    // Se outro core/task atualizou concorrentemente, reload e tenta de novo.
    let mut prev = MAX_CYCLE_US.load(Ordering::Relaxed);
    while us > prev {
        match MAX_CYCLE_US.compare_exchange(prev, us, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(p) => prev = p,
        }
    }

    if us > MAX_INPUT_CYCLE_US {
        defmt::warn!("Slow cycle: {}us (target: {}us)", us, MAX_INPUT_CYCLE_US);
    }
}

/// Reseta MAX_CYCLE_US e retorna o pico da janela anterior.
/// Usado pela diagnostic_task a cada 5s.
pub fn reset_max_cycle() -> u32 {
    MAX_CYCLE_US.swap(0, Ordering::Relaxed)
}

/// Loga estatísticas via defmt.
pub fn log_stats() {
    let sent = REPORTS_SENT.load(Ordering::Relaxed);
    let errs = SEND_ERRORS.load(Ordering::Relaxed);
    let cycles = SENSOR_CYCLES.load(Ordering::Relaxed);
    let last = LAST_CYCLE_US.load(Ordering::Relaxed);
    let max = MAX_CYCLE_US.load(Ordering::Relaxed);

    defmt::info!(
        "HID: sent={} errs={} cycles={} last={}us max={}us",
        sent,
        errs,
        cycles,
        last,
        max
    );
}
