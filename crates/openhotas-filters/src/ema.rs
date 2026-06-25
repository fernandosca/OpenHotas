use crate::tuning::DEFAULT_EMA_ALPHA;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sample_initializes() {
        let mut ema = Ema::new(0.3);
        assert_eq!(ema.apply(0.5), 0.5);
    }

    #[test]
    fn smoothing_known_alpha() {
        let mut ema = Ema::new(0.5);
        ema.apply(0.0);
        let result = ema.apply(1.0);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn converges_to_target() {
        let mut ema = Ema::new(0.3);
        ema.apply(0.0);
        for _ in 0..30 {
            ema.apply(1.0);
        }
        let result = ema.apply(1.0);
        assert!(
            (result - 1.0).abs() < 0.001,
            "EMA did not converge: {}",
            result
        );
    }

    #[test]
    fn reset_forces_reinit() {
        let mut ema = Ema::new(0.3);
        ema.apply(1.0);
        ema.reset();
        assert_eq!(ema.apply(0.0), 0.0);
    }

    #[test]
    fn alpha_zero_holds() {
        let mut ema = Ema::new(0.0);
        ema.apply(1.0);
        assert_eq!(ema.apply(2.0), 1.0);
    }

    #[test]
    fn alpha_one_passthrough() {
        let mut ema = Ema::new(1.0);
        assert_eq!(ema.apply(0.5), 0.5);
        assert_eq!(ema.apply(0.8), 0.8);
    }
}
