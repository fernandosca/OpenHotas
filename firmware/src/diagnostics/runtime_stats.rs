use crate::constants::MAX_INPUT_CYCLE_US;
use core::sync::atomic::{AtomicU32, Ordering};

pub static REPORTS_SENT: AtomicU32 = AtomicU32::new(0);
pub static SEND_ERRORS: AtomicU32 = AtomicU32::new(0);
pub static SENSOR_CYCLES: AtomicU32 = AtomicU32::new(0);
pub static LAST_CYCLE_US: AtomicU32 = AtomicU32::new(0);
pub static MAX_CYCLE_US: AtomicU32 = AtomicU32::new(0);

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

#[allow(dead_code)]
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
