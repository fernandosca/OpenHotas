use crate::constants::{MT6826_ANGLE_CENTER, MT6826_ANGLE_MAX};

#[derive(Debug, Clone, Copy)]
pub struct CalibrationData {
    pub center: u16,
    pub min: u16,
    pub max: u16,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            center: MT6826_ANGLE_CENTER,
            min: 0,
            max: MT6826_ANGLE_MAX,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Calibration {
    data: CalibrationData,
    is_calibrating: bool,
    capture_min: u16,
    capture_max: u16,
}

impl Calibration {
    pub fn new(data: CalibrationData) -> Self {
        Self {
            data,
            is_calibrating: false,
            capture_min: u16::MAX,
            capture_max: 0,
        }
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

    #[allow(dead_code)]
    pub fn start_calibration(&mut self) {
        self.is_calibrating = true;
        self.capture_min = u16::MAX;
        self.capture_max = 0;
    }

    #[allow(dead_code)]
    pub fn feed(&mut self, raw: u16) {
        if !self.is_calibrating {
            return;
        }
        if raw < self.capture_min {
            self.capture_min = raw;
        }
        if raw > self.capture_max {
            self.capture_max = raw;
        }
    }

    #[allow(dead_code)]
    pub fn finish_calibration(&mut self, center_raw: u16) -> CalibrationData {
        self.is_calibrating = false;
        CalibrationData {
            center: center_raw,
            min: self.capture_min.min(center_raw),
            max: self.capture_max.max(center_raw),
        }
    }
}
