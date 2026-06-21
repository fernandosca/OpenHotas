use crate::constants::tuning::DEFAULT_EMA_ALPHA;

#[derive(Debug, Clone)]
pub struct Ema {
    alpha: f32,
    value: f32,
    initialized: bool,
}

impl Ema {
    pub fn new(alpha: f32) -> Self {
        let clamped = alpha.clamp(0.0, 1.0);
        Self {
            alpha: clamped,
            value: 0.0,
            initialized: false,
        }
    }

    pub fn apply(&mut self, input: f32) -> f32 {
        let input = input.clamp(-1.0, 1.0);
        if !self.initialized {
            self.value = input;
            self.initialized = true;
            return input;
        }
        self.value = self.alpha * input + (1.0 - self.alpha) * self.value;
        self.value.clamp(-1.0, 1.0)
    }

    pub fn reset(&mut self) {
        self.initialized = false;
    }

    pub fn set_alpha(&mut self, a: f32) {
        self.alpha = a.clamp(0.0, 1.0);
    }
}

impl Default for Ema {
    fn default() -> Self {
        Self::new(DEFAULT_EMA_ALPHA)
    }
}
