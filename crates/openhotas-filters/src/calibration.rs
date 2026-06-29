const DEFAULT_CENTER: u16 = 16384;
const DEFAULT_MAX: u16 = 32767;
const SENSOR_COUNTS: i32 = 32768;
const HALF_TURN: i32 = SENSOR_COUNTS / 2;

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

impl CalibrationData {
    /// Shortest signed distance from `center` to `raw` on the 15-bit circle.
    /// The result is always in -16384..=16383, so crossing 32767 -> 0 is
    /// continuous for any axis whose travel is less than half a revolution.
    pub fn circular_delta(raw: u16, center: u16) -> i32 {
        ((raw as i32 - center as i32 + HALF_TURN) & (SENSOR_COUNTS - 1)) - HALF_TURN
    }

    pub fn min_delta(&self) -> i32 {
        Self::circular_delta(self.min, self.center)
    }

    pub fn max_delta(&self) -> i32 {
        Self::circular_delta(self.max, self.center)
    }

    /// Total calibrated travel in raw counts, independent of zero crossing.
    pub fn span(&self) -> u32 {
        self.min_delta().unsigned_abs() + self.max_delta().unsigned_abs()
    }

    /// Both endpoints must be non-zero, lie on opposite sides of center, and
    /// provide enough physical travel to reject accidental captures.
    pub fn is_valid(&self, minimum_span: u32) -> bool {
        let min = self.min_delta();
        let max = self.max_delta();
        min != 0 && max != 0 && min.signum() != max.signum() && self.span() >= minimum_span
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
        let delta = CalibrationData::circular_delta(raw, self.data.center);
        let min_delta = self.data.min_delta();
        let max_delta = self.data.max_delta();

        if delta == 0 || !self.data.is_valid(1) {
            return 0.0;
        }

        if delta.signum() == min_delta.signum() {
            (-(delta as f32 / min_delta as f32)).clamp(-1.0, 0.0)
        } else {
            (delta as f32 / max_delta as f32).clamp(0.0, 1.0)
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
    fn calibration_crosses_zero() {
        let cal = CalibrationData {
            min: 30000,
            center: 1000,
            max: 4000,
        };
        let wrapper = Calibration::new(cal);

        assert!(cal.is_valid(1000));
        assert_eq!(wrapper.apply(30000), -1.0);
        assert_eq!(wrapper.apply(1000), 0.0);
        assert_eq!(wrapper.apply(4000), 1.0);
        assert!(wrapper.apply(32700) < 0.0);
        assert!(wrapper.apply(2000) > 0.0);
    }

    #[test]
    fn reversed_sensor_direction_crosses_zero() {
        let cal = CalibrationData {
            min: 4000,
            center: 1000,
            max: 30000,
        };
        let wrapper = Calibration::new(cal);

        assert!(cal.is_valid(1000));
        assert_eq!(wrapper.apply(4000), -1.0);
        assert_eq!(wrapper.apply(1000), 0.0);
        assert_eq!(wrapper.apply(30000), 1.0);
    }

    #[test]
    fn endpoints_on_same_side_are_invalid() {
        let cal = CalibrationData {
            min: 1200,
            center: 1000,
            max: 4000,
        };
        assert!(!cal.is_valid(1));
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
