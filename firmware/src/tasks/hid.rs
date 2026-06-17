use crate::diagnostics::runtime_stats;
use crate::usb::hid_gamepad::{UsbDriver, REPORT_SIGNAL};
use embassy_usb::class::hid::HidWriter;
use embassy_usb::UsbDevice;

#[embassy_executor::task]
pub async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    loop {
        device.run().await;
    }
}

#[embassy_executor::task]
pub async fn hid_task(mut writer: HidWriter<'static, UsbDriver, 64>) -> ! {
    loop {
        let report = REPORT_SIGNAL.wait().await;
        if writer.write(&report.to_bytes()).await.is_err() {
            runtime_stats::record_send_error();
        } else {
            runtime_stats::record_report_sent();
        }
    }
}
