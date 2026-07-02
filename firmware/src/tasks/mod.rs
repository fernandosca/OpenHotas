//! Tasks assíncronas do firmware (embassy-executor).
//!
//! - `input`: task principal, 500μs, lê sensores → pipeline → HID report.
//! - `hid`: gerencia o barramento USB + envia HID reports.
//! - `cdc`: protocolo binário request/response (config, calibração).
//! - `cdc_handlers`: dispatch de requests do protocolo.
//! - `diagnostic`: log periódico de runtime stats via defmt.

pub mod cdc;
pub mod cdc_handlers;
pub mod diagnostic;
pub mod hid;
pub mod input;
