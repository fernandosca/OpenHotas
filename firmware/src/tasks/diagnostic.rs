//! Diagnostic task — logs RuntimeStats via defmt a cada 5s.
//!
//! V1.22: desacoplada do CDC. O CDC agora é reservado para protocolo
//! binário request/response. Logs textuais espontâneos no CDC quebrariam
//! o parser do protocolo.

use crate::diagnostics::runtime_stats;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn diagnostic_task() -> ! {
    loop {
        let peak = runtime_stats::reset_max_cycle();
        runtime_stats::log_stats();
        if peak > 0 {
            defmt::info!("Cycle peak this window: {}us", peak);
        }
        Timer::after_secs(5).await;
    }
}
