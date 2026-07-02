//! Configuração do dispositivo: runtime (viva) e persistente (flash).
//!
//! - `runtime`: tipos leves para comunicação cdc_task → input_task via Signal.
//! - `stored_config_v2`: persistência em flash com double-buffer power-fail safe.

pub mod runtime;
pub mod stored_config_v2;
