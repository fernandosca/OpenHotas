use crate::axis::AxisOutput;
use crate::constants::{HID_AXIS_MAX, REPORT_SIZE};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub type UsbDriver = embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>;

/// # Safety
/// `v` é clampado em [-1.0, 1.0] antes desta chamada, então o cast `as i16`
/// é seguro: o resultado está sempre em [-32767, +32767].
pub fn axis_to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * HID_AXIS_MAX as f32) as i16
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GamepadReport {
    pub x: AxisOutput,
    pub y: AxisOutput,
    pub twist: AxisOutput,
    pub buttons: u32,
}

impl GamepadReport {
    pub fn to_bytes(self) -> [u8; REPORT_SIZE] {
        let x = axis_to_i16(self.y.value).to_le_bytes();
        let y = axis_to_i16(self.x.value).to_le_bytes();
        let rx = axis_to_i16(self.twist.value).to_le_bytes();
        let b = self.buttons.to_le_bytes();

        [x[0], x[1], y[0], y[1], rx[0], rx[1], b[0], b[1], b[2], b[3]]
    }
}

pub static REPORT_SIGNAL: Signal<CriticalSectionRawMutex, GamepadReport> = Signal::new();
