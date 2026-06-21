use super::{AxisConfig, AxisOutput};
use crate::calibration::data::{Calibration, CalibrationData};
use crate::filters::deadzone::Deadzone;
use crate::filters::ema::Ema;
use crate::filters::expo::Expo;
use crate::filters::max_jump::MaxJump;
use crate::filters::response_curve::ResponseCurve;

#[derive(Debug)]
pub struct AxisPipeline {
    calibration: Calibration,
    max_jump: MaxJump,
    ema: Ema,
    deadzone: Deadzone,
    expo: Expo,
    response: ResponseCurve,
    config: AxisConfig,
}

impl AxisPipeline {
    pub fn new(cal_data: CalibrationData, config: AxisConfig) -> Self {
        Self {
            calibration: Calibration::new(cal_data),
            max_jump: MaxJump::new(config.max_jump),
            ema: Ema::new(config.ema_alpha),
            deadzone: Deadzone::new(config.deadzone),
            expo: Expo::new(config.expo),
            response: ResponseCurve::new(),
            config,
        }
    }

    pub fn process(&mut self, raw: u16, healthy: bool) -> AxisOutput {
        let cal = self.calibration.apply(raw);
        let safe = self.max_jump.apply(cal);
        let smt = self.ema.apply(safe);

        let (dz, reset_ema) = self.deadzone.apply(smt);
        if reset_ema {
            self.ema.reset();
        }

        let exp = self.expo.apply(dz);
        let out = self.response.apply(exp);

        let value = if self.config.inverted { -out } else { out };

        AxisOutput {
            value: value.clamp(-1.0, 1.0),
            healthy,
        }
    }

    #[allow(dead_code)]
    pub fn update_config(&mut self, cfg: AxisConfig) {
        self.max_jump.set_threshold(cfg.max_jump);
        self.ema.set_alpha(cfg.ema_alpha);
        self.deadzone.set_threshold(cfg.deadzone);
        self.expo.set_factor(cfg.expo);
        self.config = cfg;
    }
}
