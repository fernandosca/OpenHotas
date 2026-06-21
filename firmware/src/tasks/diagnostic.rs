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
        runtime_stats::log_stats();
        Timer::after_secs(5).await;
    }
}
