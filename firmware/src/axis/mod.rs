pub mod pipeline;

pub use pipeline::AxisPipeline;

use crate::constants::tuning::{
    DEFAULT_DEADZONE, DEFAULT_EMA_ALPHA, DEFAULT_EXPO, DEFAULT_MAX_JUMP,
};

#[derive(Debug, Clone, Copy)]
pub struct AxisConfig {
    pub ema_alpha: f32,
    pub deadzone: f32,
    pub max_jump: f32,
    pub expo: f32,
    pub inverted: bool,
    pub reset_ema_on_dz: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            ema_alpha: DEFAULT_EMA_ALPHA,
            deadzone: DEFAULT_DEADZONE,
            max_jump: DEFAULT_MAX_JUMP,
            expo: DEFAULT_EXPO,
            inverted: false,
            reset_ema_on_dz: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AxisOutput {
    pub value: f32,
    #[allow(dead_code)]
    pub healthy: bool,
}
