//! Gamepad HID report e canal de comunicação com a task USB.
//!
//! # Arquitetura
//!
//! input_task produz `GamepadReport` a cada 500μs e o envia via `REPORT_SIGNAL`.
//! hid_task (em tasks/hid.rs) aguarda o sinal e escreve no HID endpoint USB.
//!
//! `REPORT_SIGNAL` é um `Signal` (capacity=1, lossy). Se input_task produzir
//! dois reports antes de hid_task consumir, o mais antigo é sobrescrito.
//! Isso é INTENCIONAL — para joystick, o host sempre quer o estado mais
//! recente, não um buffer de históricos.
//!
//! # HID Report Descriptor
//!
//! O descritor (em descriptor.rs) declara X, Y, Rx como signed i16 e
//! 32 botões como 1-bit cada. Total: 10 bytes.

use crate::axis::AxisOutput;
use crate::constants::{HID_AXIS_MAX, REPORT_SIZE};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub type UsbDriver = embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>;

/// Converte valor normalizado [-1.0, 1.0] para i16 do HID.
///
/// # Safety
/// `v` é clampado em [-1.0, 1.0] e HID_AXIS_MAX = 32767, então
/// `clamp * 32767` está sempre em [-32767.0, 32767.0], seguro para `as i16`.
/// Não há overflow mesmo nos extremos.
pub fn axis_to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * HID_AXIS_MAX as f32) as i16
}

/// Report HID do gamepad.
///
/// `x`/`y`/`twist` seguem a nomenclatura do pipeline do firmware (X = roll,
/// Y = pitch, Twist = yaw). A ordem de bytes no HID report segue o descritor:
/// X, Y, Rx.
#[derive(Debug, Default, Clone, Copy)]
pub struct GamepadReport {
    pub x: AxisOutput,
    pub y: AxisOutput,
    pub twist: AxisOutput,
    pub buttons: u32,
}

impl GamepadReport {
    /// Serializa para o formato binário de 10 bytes do HID.
    ///
    /// Ordem: X (i16 LE), Y (i16 LE), Rx (i16 LE), Buttons (u32 LE).
    pub fn to_bytes(self) -> [u8; REPORT_SIZE] {
        let x = axis_to_i16(self.x.value).to_le_bytes();
        let y = axis_to_i16(self.y.value).to_le_bytes();
        let rx = axis_to_i16(self.twist.value).to_le_bytes();
        let b = self.buttons.to_le_bytes();

        [x[0], x[1], y[0], y[1], rx[0], rx[1], b[0], b[1], b[2], b[3]]
    }
}

/// Canal lossy de GamepadReport: input_task → hid_task.
/// Capacity=1 (Signal, não Channel). O report mais recente sempre substitui
/// o anterior — não há fila.
pub static REPORT_SIGNAL: Signal<CriticalSectionRawMutex, GamepadReport> = Signal::new();
