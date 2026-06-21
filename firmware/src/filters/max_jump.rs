use crate::constants::tuning::DEFAULT_MAX_JUMP;
use libm::fabsf;

#[derive(Debug, Clone)]
pub struct MaxJump {
    last_valid: f32,
    initialized: bool,
    threshold: f32,
}

impl MaxJump {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: fabsf(threshold),
            last_valid: 0.0,
            initialized: false,
        }
    }

    pub fn apply(&mut self, input: f32) -> f32 {
        if !self.initialized {
            self.last_valid = input;
            self.initialized = true;
            return input.clamp(-1.0, 1.0);
        }

        let delta = fabsf(input - self.last_valid);
        if delta > self.threshold {
            return self.last_valid;
        }

        self.last_valid = input;
        input.clamp(-1.0, 1.0)
    }

    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = fabsf(t);
    }

    /// V1.25: reset internal state — forces re-initialization on next apply().
    /// Called when calibration, travel limits, or enabled status changes.
    pub fn reset(&mut self) {
        self.initialized = false;
    }
}

impl Default for MaxJump {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_JUMP)
    }
}
