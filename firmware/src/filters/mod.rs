//! Re-export dos filtros de sinal da crate `openhotas-filters`.
//!
//! Disponibiliza os filtros como `crate::filters::*` para o pipeline
//! em `axis::pipeline`. Cada filtro é uma struct independente, sem trait comum.
//!
//! Pipeline order: MaxJump → Ema → Deadzone → ResponseCurve
//! (ver docs individuais de cada filtro para rationale da ordem).

#[allow(unused_imports)]
pub use openhotas_filters::deadzone;
#[allow(unused_imports)]
pub use openhotas_filters::ema;
#[allow(unused_imports)]
pub use openhotas_filters::max_jump;
#[allow(unused_imports)]
pub use openhotas_filters::response_curve;
#[allow(unused_imports)]
pub use openhotas_filters::{Deadzone, Ema, MaxJump, ResponseCurve};
