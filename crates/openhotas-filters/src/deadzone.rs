use crate::tuning::DEFAULT_DEADZONE;
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

        if self.threshold >= 1.0 {
            return (input, just_entered);
        }

        let sign = if input >= 0.0 { 1.0 } else { -1.0 };
        let val = sign * (fabsf(input) - self.threshold) / (1.0 - self.threshold);
        (val, just_entered)
    }

    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
    }
}

impl Default for Deadzone {
    fn default() -> Self {
        Self::new(DEFAULT_DEADZONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_zeroed() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(0.05);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn above_threshold_remapped() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(0.55);
        assert!((out - 0.5).abs() < 0.001);
    }

    #[test]
    fn just_entered_flag() {
        let mut dz = Deadzone::new(0.1);
        dz.apply(0.5);
        let (_, entered) = dz.apply(0.05);
        assert!(entered);
    }

    #[test]
    fn negative_side_mirrors() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(-0.55);
        assert!((out - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn zero_threshold_passthrough() {
        let mut dz = Deadzone::new(0.0);
        let (out, _) = dz.apply(0.75);
        assert!((out - 0.75).abs() < 0.001);
    }
}
