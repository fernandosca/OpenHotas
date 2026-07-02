//! Valores padrão para filtros do pipeline de sinal.
//!
//! Separados das constantes de hardware (em `firmware/src/constants.rs`)
//! para isolar parâmetros ajustáveis em campo dos valores de contrato fixo.
//! Estes defaults são sobrescritos quando uma configuração é carregada da flash.
//!
//! - DEFAULT_EMA_ALPHA = 0.3: suavização moderada (compromisso ruído vs latência).
//! - DEFAULT_DEADZONE = 0.02 (2%): zona morta pequena, joystick novo sem folga.
//! - DEFAULT_MAX_JUMP = 0.15: rejeita spikes > 15% da faixa entre amostras consecutivas.

/// Fator de suavização EMA padrão (0.3 = moderado).
pub const DEFAULT_EMA_ALPHA: f32 = 0.3;
/// Limiar de zona morta padrão (2% da faixa).
pub const DEFAULT_DEADZONE: f32 = 0.02;
/// Limiar de rejeição de spike padrão (15% entre amostras consecutivas).
pub const DEFAULT_MAX_JUMP: f32 = 0.15;
