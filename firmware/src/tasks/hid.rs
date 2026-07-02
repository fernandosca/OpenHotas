//! Tasks de USB HID: gerenciamento do barramento + envio de reports.
//!
//! # Por que duas tasks separadas?
//!
//! O embassy-usb exige que `UsbDevice::run()` seja chamado em loop para
//! manter o barramento USB vivo (poll de SOF, reset, etc.). Essa task
//! (`usb_task`) não pode ser interrompida por longos períodos.
//!
//! Separar o envio de HID reports em outra task (`hid_task`) permite que:
//! - O barramento USB seja servido mesmo se o HID endpoint estiver ocupado.
//! - O envio de reports use `REPORT_SIGNAL.wait()` (async, sem polling).
//!
//! A comunicação entre tasks é via `REPORT_SIGNAL` (Signal lossy),
//! definido em `usb/hid_gamepad.rs`.

use crate::diagnostics::runtime_stats;
use crate::usb::hid_gamepad::{UsbDriver, REPORT_SIGNAL};
use embassy_usb::class::hid::HidWriter;
use embassy_usb::UsbDevice;

/// Mantém o barramento USB ativo.
/// Deve rodar em loop — nunca retorna.
#[embassy_executor::task]
pub async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    loop {
        device.run().await;
    }
}

/// Aguarda reports do input_task e os envia pelo HID endpoint.
///
/// Se a escrita falhar (USB desconectado, buffer cheio), incrementa
/// o contador de erros (visível via GetRuntimeStats no protocolo CDC).
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
