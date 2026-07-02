//! Tipos de configuração e saída dos eixos do joystick.
//!
//! `AxisConfig` contém os parâmetros dos filtros (valores normalizados f32).
//! É distinto de `AxisRuntimeConfig` (em `config::runtime`), que adiciona
//! calibração, travel limits e axis-to-button — a separação existe porque
//! `AxisConfig` é o subconjunto de filtros que pode ser reutilizado sem
//! depender do runtime config completo.

pub mod pipeline;

pub use pipeline::AxisPipeline;

use crate::constants::tuning::{DEFAULT_DEADZONE, DEFAULT_EMA_ALPHA, DEFAULT_MAX_JUMP};

/// Parâmetros dos filtros do pipeline de sinal (valores normalizados f32).
///
/// Estes valores são convertidos de permille do protocolo em `from_protocol_config`.
/// Ver `config::runtime::AxisRuntimeConfig` para a configuração completa do eixo.
#[derive(Debug, Clone, Copy)]
pub struct AxisConfig {
    pub ema_alpha: f32,
    pub deadzone: f32,
    pub max_jump: f32,
    /// Inverte a direção do eixo (multiplica saída por -1).
    pub inverted: bool,
    /// Se `true`, o EMA é resetado ao entrar na zona morta.
    /// Habilitado apenas para o eixo Twist (default).
    pub reset_ema_on_dz: bool,
    /// Ponto de controle P1 da ResponseCurve (lado negativo).
    pub response_p1: (f32, f32),
    /// Ponto de controle P3 da ResponseCurve (lado positivo).
    pub response_p3: (f32, f32),
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            ema_alpha: DEFAULT_EMA_ALPHA,
            deadzone: DEFAULT_DEADZONE,
            max_jump: DEFAULT_MAX_JUMP,
            inverted: false,
            reset_ema_on_dz: false,
            response_p1: (-0.5, -0.5),
            response_p3: (0.5, 0.5),
        }
    }
}

/// Saída processada de um eixo, após passar por todo o pipeline.
#[derive(Debug, Clone, Copy, Default)]
pub struct AxisOutput {
    /// Valor normalizado [-1.0, 1.0] após calibração + filtros.
    pub value: f32,
    /// `true` se o sensor subjacente está produzindo leituras válidas.
    /// `false` indica CRC error, magnet error ou sensor ausente.
    pub healthy: bool,
}
