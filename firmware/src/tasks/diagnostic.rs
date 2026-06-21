use crate::constants::{
    DIAGNOSTIC_INTERVAL_SECS, FIRMWARE_GIT_HASH, FIRMWARE_VERSION, MAX_INPUT_CYCLE_US,
};
use crate::diagnostics::runtime_stats;

use core::sync::atomic::Ordering;

use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::Sender;
use embassy_usb::driver::EndpointError;

use ufmt::uwrite;

/// Buffer de formatação CDC — implementa uWrite para uso com ufmt.
struct WriteCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
    overflow: bool,
}

impl<'a> WriteCursor<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            overflow: false,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    fn overflowed(&self) -> bool {
        self.overflow
    }
}

impl ufmt::uWrite for WriteCursor<'_> {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let bytes = s.as_bytes();

        if self.pos + bytes.len() > self.buf.len() {
            self.overflow = true;
            return Err(());
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();

        Ok(())
    }
}

/// Envia dados via CDC em chunks de 64 bytes (limite USB Full Speed).
async fn send_text(
    cdc: &mut Sender<'static, Driver<'static, USB>>,
    data: &[u8],
) -> Result<(), EndpointError> {
    for chunk in data.chunks(64) {
        cdc.write_packet(chunk).await?;
    }
    Ok(())
}

#[embassy_executor::task]
pub async fn diagnostic_task(mut cdc: Sender<'static, Driver<'static, USB>>) -> ! {
    loop {
        cdc.wait_connection().await;

        // Banner de conexão com versão do firmware
        if send_text(&mut cdc, b"OpenHOTAS CDC Debug Connected\r\n")
            .await
            .is_err()
        {
            continue;
        }

        let mut buf = [0u8; 128];
        let mut w = WriteCursor::new(&mut buf);
        let _ = uwrite!(
            w,
            "{} (git:{})\r\n\r\n",
            FIRMWARE_VERSION,
            FIRMWARE_GIT_HASH
        );
        if send_text(&mut cdc, w.as_bytes()).await.is_err() {
            continue;
        }

        // Loop de telemetria
        loop {
            let cycles = runtime_stats::SENSOR_CYCLES.load(Ordering::Relaxed);
            let max_us = runtime_stats::MAX_CYCLE_US.load(Ordering::Relaxed);
            let last_us = runtime_stats::LAST_CYCLE_US.load(Ordering::Relaxed);
            let reports = runtime_stats::REPORTS_SENT.load(Ordering::Relaxed);
            let errors = runtime_stats::SEND_ERRORS.load(Ordering::Relaxed);

            let mut buf = [0u8; 128];
            let mut w = WriteCursor::new(&mut buf);

            let _ = uwrite!(w, "cycles={}\r\n", cycles);
            let _ = uwrite!(w, "last={}\r\n", last_us);
            let _ = uwrite!(w, "max={}\r\n", max_us);
            let _ = uwrite!(w, "reports={}\r\n", reports);
            let _ = uwrite!(w, "errors={}\r\n", errors);

            if max_us > MAX_INPUT_CYCLE_US {
                let _ = uwrite!(w, "WARN:max_cycle\r\n");
            }

            if send_text(&mut cdc, w.as_bytes()).await.is_err() {
                break;
            }

            if w.overflowed() && send_text(&mut cdc, b"[diag truncated]\r\n").await.is_err() {
                break;
            }

            Timer::after_secs(DIAGNOSTIC_INTERVAL_SECS).await;
        }
    }
}
