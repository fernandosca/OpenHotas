use crate::constants::tuning::DEFAULT_DEADZONE;
use libm::fabsf;

#[derive(Debug)]
pub struct Deadzone {
    threshold: f32,
    in_zone: bool,
}

impl Deadzone {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            in_zone: false,
        }
    }

    pub fn apply(&mut self, input: f32) -> (f32, bool) {
        let input = input.clamp(-1.0, 1.0);
        let mut just_entered = false;

        if fabsf(input) < self.threshold {
            if !self.in_zone {
                just_entered = true;
            }
            self.in_zone = true;
            return (0.0, just_entered);
        }

        self.in_zone = false;

        let sign = if input >= 0.0 { 1.0 } else { -1.0 };
        let val = sign * (fabsf(input) - self.threshold) / (1.0 - self.threshold);
        (val, just_entered)
    }

    #[allow(dead_code)]
    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
    }
}

impl Default for Deadzone {
    fn default() -> Self {
        Self::new(DEFAULT_DEADZONE)
    }
}
