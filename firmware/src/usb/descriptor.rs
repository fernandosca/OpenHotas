//! Descritor HID do report de gamepad.
//!
//! Report de 10 bytes:
//! - bytes 0-5: 3 eixos analógicos (X, Y, Rx) como signed i16 LE (-32767..+32767)
//! - bytes 6-9: 32 botões como 32x 1-bit
//!
//! NOTA: X (HID) = pitch (eixo Y do firmware), Y (HID) = roll (eixo X).
//! A troca é feita em `hid_gamepad::GamepadReport::to_bytes()`.
//! Ver lá para a rationale.
//!
//! Report ID não usado (V1.22): só existe 1 report HID no dispositivo.
//! Adicionar Report ID quebraria compatibilidade com hosts que esperam
//! report puro sem byte de ID.

pub const HID_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    // Eixos X, Y, Rx (Twist) — 3x signed i16
    // Report ID removido: V1.22 só tem 1 report HID — não necessário
    0x09, 0x30, // Usage (X)
    0x09, 0x31, // Usage (Y)
    0x09, 0x33, // Usage (Rx)
    0x16, 0x01, 0x80, // Logical Minimum (-32767) — eixos assinados
    0x26, 0xFF, 0x7F, // Logical Maximum (32767)
    0x36, 0x01, 0x80, // Physical Minimum (-32767) — pré-calibrado, sem range oculto
    0x46, 0xFF, 0x7F, // Physical Maximum (32767)
    0x75, 0x10, // Report Size (16)
    0x95, 0x03, // Report Count (3)
    0x81, 0x02, // Input (Data, Var, Abs)
    // Botoes 1..32 — 32x 1-bit
    0x05, 0x09, // Usage Page (Button)
    0x19, 0x01, // Usage Minimum (1)
    0x29, 0x20, // Usage Maximum (32)
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x01, // Logical Maximum (1)
    0x35, 0x00, // Physical Minimum (0)
    0x45, 0x01, // Physical Maximum (1)
    0x75, 0x01, // Report Size (1)
    0x95, 0x20, // Report Count (32)
    0x81, 0x02, // Input (Data, Var, Abs)
    0xC0, // End Collection
];
