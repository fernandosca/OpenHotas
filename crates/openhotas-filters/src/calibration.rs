const DEFAULT_CENTER: u16 = 16384;
const DEFAULT_MAX: u16 = 32767;

#[derive(Debug, Clone, Copy)]
pub struct CalibrationData {
    pub center: u16,
    pub min: u16,
    pub max: u16,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            center: DEFAULT_CENTER,
            min: 0,
            max: DEFAULT_MAX,
        }
    }
}

/// Runtime calibration wrapper — applies CalibrationData to raw u16 samples.
#[derive(Debug)]
pub struct Calibration {
    pub data: CalibrationData,
}

impl Calibration {
    pub fn new(data: CalibrationData) -> Self {
        Self { data }
    }

    pub fn apply(&self, raw: u16) -> f32 {
        let raw = raw.clamp(self.data.min, self.data.max);

        if raw <= self.data.center {
            let range = (self.data.center - self.data.min) as f32;
            if range == 0.0 {
                return 0.0;
            }
            let result = -((self.data.center - raw) as f32) / range;
            result.clamp(-1.0, 1.0)
        } else {
            let range = (self.data.max - self.data.center) as f32;
            if range == 0.0 {
                return 0.0;
            }
            let result = (raw - self.data.center) as f32 / range;
            result.clamp(-1.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_returns_zero() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(16384), 0.0);
    }

    #[test]
    fn min_returns_neg_one() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(0), -1.0);
    }

    #[test]
    fn max_returns_pos_one() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(32767), 1.0);
    }

    #[test]
    fn degenerate_range_no_panic() {
        let cal = CalibrationData {
            center: 16384,
            min: 16384,
            max: 16384,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(16384), 0.0);
    }

    #[test]
    fn asymmetric_sides() {
        let cal = CalibrationData {
            center: 20000,
            min: 10000,
            max: 30000,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(10000), -1.0);
        assert_eq!(wrapper.apply(30000), 1.0);
    }

    #[test]
    fn symmetric_center() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert!((wrapper.apply(8192) - (-0.5)).abs() < 0.001);
        assert!((wrapper.apply(24576) - 0.5).abs() < 0.001);
    }
}
