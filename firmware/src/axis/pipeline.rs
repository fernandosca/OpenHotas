use super::{AxisConfig, AxisOutput};
use crate::calibration::data::{Calibration, CalibrationData};
use crate::config::runtime::{AxisRuntimeConfig, AxisTravelLimitsRuntime};
use crate::filters::deadzone::Deadzone;
use crate::filters::ema::Ema;
use crate::filters::max_jump::MaxJump;
use crate::filters::response_curve::ResponseCurve;

#[derive(Debug)]
pub struct AxisPipeline {
    calibration: Calibration,
    max_jump: MaxJump,
    ema: Ema,
    deadzone: Deadzone,
    response: ResponseCurve,
    config: AxisConfig,
    /// Travel limits applied after calibration, before filters.
    /// Scales the normalized [-1,1] signal to the configured range.
    travel: AxisTravelLimitsRuntime,
    enabled: bool,
    /// Center offset in f32 (-0.2 to 0.2). Applied after calibration.
    center_offset: f32,
}

impl AxisPipeline {
    pub fn new(cal_data: CalibrationData, config: AxisConfig) -> Self {
        Self {
            calibration: Calibration::new(cal_data),
            max_jump: MaxJump::new(config.max_jump),
            ema: Ema::new(config.ema_alpha),
            deadzone: Deadzone::new(config.deadzone),
            response: ResponseCurve::new(config.response_p1, config.response_p3),
            config,
            travel: AxisTravelLimitsRuntime::default(),
            enabled: true,
            center_offset: 0.0,
        }
    }

    /// Process a raw sensor sample through the full pipeline.
    ///
    /// Pipeline order (V1.3):
    /// Raw → Calibration → Center Offset → Travel Limits → MaxJump → EMA → Deadzone → Response
    pub fn process(&mut self, raw: u16, healthy: bool) -> AxisOutput {
        // Disabled axis: output centered, reported as healthy
        if !self.enabled {
            return AxisOutput {
                value: 0.0,
                healthy: true,
            };
        }

        let cal = self.calibration.apply(raw);

        // Apply center offset (V1.3 — after calibration, before travel limits)
        let cal = (cal + self.center_offset).clamp(-1.0, 1.0);

        // Apply travel limits (V1.22 — after calibration, before filters)
        let cal = self.apply_travel_limits(cal);

        let safe = self.max_jump.apply(cal);
        let smt = self.ema.apply(safe);

        let (dz, reset_ema) = self.deadzone.apply(smt);
        // Only reset EMA on deadzone entry if axis config allows it (e.g. Twist axis).
        // X/Y axes should NOT reset EMA — preserves filter continuity near center.
        if reset_ema && self.config.reset_ema_on_dz {
            self.ema.reset();
        }

        let out = self.response.apply(dz);

        let value = if self.config.inverted { -out } else { out };

        AxisOutput {
            value: value.clamp(-1.0, 1.0),
            healthy,
        }
    }

    /// Apply travel limits to a normalized [-1, 1] signal.
    ///
    /// Travel limits solve: "reached physical end but PC shows <100%".
    /// `travel_limit_permille` defines how far from center each side must move
    /// before saturating to full output.
    ///
    /// Example: 950 means ±95% travel remaps to full [-1.0, 1.0] output.
    fn apply_travel_limits(&self, input: f32) -> f32 {
        let limit = self.travel.travel_limit_permille as f32 / 1000.0;
        if limit <= 0.0 {
            return input;
        }
        (input / limit).clamp(-1.0, 1.0)
    }

    /// Apply new runtime configuration (from CONFIG_SIGNAL).
    ///
    /// V1.22: replaces the old `update_config(AxisConfig)` stub.
    /// Called by input_task when new config arrives.
    pub fn update_runtime_config(&mut self, cfg: AxisRuntimeConfig) {
        let was_disabled = !self.enabled;
        let cal_changed = self.calibration.data.min != cfg.calibration.min
            || self.calibration.data.center != cfg.calibration.center
            || self.calibration.data.max != cfg.calibration.max;

        self.enabled = cfg.enabled;
        // Update calibration if changed (e.g., after calibration via CDC)
        self.calibration = Calibration::new(cfg.calibration);
        let travel_changed = self.travel.travel_limit_permille != cfg.travel.travel_limit_permille;
        self.travel = cfg.travel;

        self.max_jump.set_threshold(cfg.filters.max_jump);
        self.ema.set_alpha(cfg.filters.ema_alpha);
        self.deadzone.set_threshold(cfg.filters.deadzone);
        self.response
            .set_points(cfg.filters.response_p1, cfg.filters.response_p3);
        self.config = cfg.filters;
        self.config.inverted = cfg.inverted;
        self.config.reset_ema_on_dz = cfg.reset_ema_on_dz;
        self.center_offset = cfg.center_offset;

        // V1.25: reset EMA and MaxJump when calibration, travel, or
        // enable-status changes — stale history invalidates filter state.
        if cal_changed || travel_changed || (was_disabled && cfg.enabled) {
            self.ema.reset();
            self.max_jump.reset();
        }
    }
}
