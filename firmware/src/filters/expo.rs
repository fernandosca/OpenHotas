use crate::constants::tuning::DEFAULT_EXPO;

#[derive(Debug, Clone)]
pub struct Expo {
    factor: f32,
}

impl Expo {
    pub fn new(factor: f32) -> Self {
        Self {
            factor: factor.clamp(0.0, 1.0),
        }
    }

    pub fn apply(&self, input: f32) -> f32 {
        let input = input.clamp(-1.0, 1.0);
        input * (1.0 - self.factor) + input * input * input * self.factor
    }

    #[allow(dead_code)]
    pub fn set_factor(&mut self, f: f32) {
        self.factor = f.clamp(0.0, 1.0);
    }
}

impl Default for Expo {
    fn default() -> Self {
        Self::new(DEFAULT_EXPO)
    }
}
