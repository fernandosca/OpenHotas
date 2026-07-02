//! Re-export dos tipos de calibração da crate `openhotas-filters`.
//!
//! `data` é um alias para o módulo `calibration` da crate, exportado como
//! `crate::calibration::data` para acesso aos tipos `CalibrationData`.

// #allow(unused_imports): estas re-exportações são usadas por outros módulos
// via `crate::calibration::*`. O lint vê como não usadas no módulo atual.
#[allow(unused_imports)]
pub use openhotas_filters::calibration as data;
#[allow(unused_imports)]
pub use openhotas_filters::{Calibration, CalibrationData};
