use crate::constants::MAX_INPUT_CYCLE_US;
use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU8, Ordering};

pub static REPORTS_SENT: AtomicU32 = AtomicU32::new(0);
pub static SEND_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_CYCLES: AtomicU32 = AtomicU32::new(0);
pub static LAST_CYCLE_US: AtomicU32 = AtomicU32::new(0);
pub static MAX_CYCLE_US: AtomicU32 = AtomicU32::new(0);

// ── V1.22: New atomics for CDC diagnostics ─────────────────────────
pub static RAW_AXIS_X: AtomicU32 = AtomicU32::new(0);
pub static RAW_AXIS_Y: AtomicU32 = AtomicU32::new(0);
pub static RAW_AXIS_TWIST: AtomicU32 = AtomicU32::new(0);

/// V1.25: AtomicI32 — processed axis values are signed (-32767..+32767).
pub static PROC_AXIS_X: AtomicI32 = AtomicI32::new(0);
pub static PROC_AXIS_Y: AtomicI32 = AtomicI32::new(0);
pub static PROC_AXIS_TWIST: AtomicI32 = AtomicI32::new(0);

pub static BUTTON_MASK: AtomicU32 = AtomicU32::new(0);

/// Bitmask: bit0=X unhealthy, bit1=Y, bit2=Twist
pub static SENSOR_UNHEALTHY: AtomicU8 = AtomicU8::new(0);

/// V1.23: protocol CRC errors (CDC frame CRC16-CCITT mismatch).
pub static PROTOCOL_CRC_ERRORS: AtomicU32 = AtomicU32::new(0);
/// V1.23: sensor CRC errors (MT6826S encoder CRC8 mismatch).
pub static SENSOR_CRC_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static MAGNET_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static FLASH_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static BUTTON_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static BUTTONS_DEGRADED: AtomicU8 = AtomicU8::new(0);

// ── V1.22: per-sensor error counters ───────────────────────────────
pub static SENSOR_X_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_Y_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_TWIST_ERRORS: AtomicU32 = AtomicU32::new(0);

pub fn record_report_sent() {
    REPORTS_SENT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_send_error() {
    SEND_ERRORS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_cycle(us: u32) {
    SENSOR_CYCLES.fetch_add(1, Ordering::Relaxed);
    LAST_CYCLE_US.store(us, Ordering::Relaxed);

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

/// Reset MAX_CYCLE_US and return the peak value from the previous window.
pub fn reset_max_cycle() -> u32 {
    MAX_CYCLE_US.swap(0, Ordering::Relaxed)
}

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
