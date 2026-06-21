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

/// Runtime calibration wrapper — applies CalibrationData to raw u16 samples.
///
/// V1.22: calibration session removed (replaced by CDC CaptureCalibrationPoint).
/// Only `apply()` is needed; calibration data updates come via
/// `AxisPipeline::update_runtime_config()` from cdc_task.
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
