use core::sync::atomic::Ordering;

use crate::axis::AxisPipeline;
use crate::config::runtime::{AxisToButtonRuntime, ButtonRuntimeConfig, CONFIG_SIGNAL};
use crate::constants::{MCP23S17_DEBOUNCE_COUNT, MT6826_ANGLE_CENTER, MT6826_POWER_UP_MS};
use crate::diagnostics::runtime_stats;
use crate::sensors::mcp23s::Mcp23s;
use crate::sensors::mt6826::Mt6826;
use crate::sensors::Sensor;
use crate::usb::hid_gamepad::{GamepadReport, REPORT_SIGNAL};
use embassy_time::{Duration, Ticker, Timer};

/// Track delta of a cumulative sensor counter into a runtime_stats atomic.
/// V1.24: replaces repetitive if-blocks (audit #6).
macro_rules! track_delta {
    ($counter:expr, $current:expr, $prev:expr) => {
        if $current > $prev {
            $counter.fetch_add($current - $prev, Ordering::Relaxed);
            $prev = $current;
        }
    };
}

/// Apply axis-to-button mapping: set bit in button mask if axis exceeds threshold.
fn apply_axis_to_button(value: f32, config: &AxisToButtonRuntime, buttons: &mut u32) {
    if !config.enabled || config.button_index > 31 {
        return;
    }

    let activate = match config.direction {
        openhotas_protocol::config::AxisDirection::Positive => value > config.threshold,
        openhotas_protocol::config::AxisDirection::Negative => value < -config.threshold,
        openhotas_protocol::config::AxisDirection::Both => libm::fabsf(value) > config.threshold,
    };

    if activate {
        *buttons |= 1u32 << config.button_index;
    }
}

#[embassy_executor::task]
pub async fn input_task(
    mut sens_x: Mt6826<'static>,
    mut sens_y: Mt6826<'static>,
    mut sens_t: Mt6826<'static>,
    mut mcp: Mcp23s<'static>,
    mut pl_x: AxisPipeline,
    mut pl_y: AxisPipeline,
    mut pl_t: AxisPipeline,
) -> ! {
    // O MT6826S especifica TPwrUp tipico de 3 ms. Todos os CS ja estao altos;
    // aguarde uma unica vez antes de iniciar qualquer transacao SPI.
    Timer::after(Duration::from_millis(MT6826_POWER_UP_MS)).await;

    // Runtime button config — starts with defaults, updated via CONFIG_SIGNAL
    let mut btn_cfg = ButtonRuntimeConfig::default();
    // Track error counts to detect per-sensor changes
    let mut prev_err_x: u32 = 0;
    let mut prev_err_y: u32 = 0;
    let mut prev_err_t: u32 = 0;
    // V1.23: separate CRC vs magnet tracking
    let mut prev_crc_x: u32 = 0;
    let mut prev_crc_y: u32 = 0;
    let mut prev_crc_t: u32 = 0;
    let mut prev_mag_x: u32 = 0;
    let mut prev_mag_y: u32 = 0;
    let mut prev_mag_t: u32 = 0;
    // Debounce threshold applied flag
    let mut debounce_applied = false;
    // Axis-to-button configs (updated from CONFIG_SIGNAL)
    let mut atb_x = AxisToButtonRuntime::default();
    let mut atb_y = AxisToButtonRuntime::default();
    let mut atb_t = AxisToButtonRuntime::default();
    let mut ticker = Ticker::every(Duration::from_micros(500));

    loop {
        // Check for new runtime config from cdc_task (non-blocking — D-07)
        if let Ok(cfg) = CONFIG_SIGNAL.try_receive() {
            pl_x.update_runtime_config(cfg.axes[0]);
            pl_y.update_runtime_config(cfg.axes[1]);
            pl_t.update_runtime_config(cfg.axes[2]);

            // Store axis-to-button configs
            atb_x = cfg.axes[0].axis_to_button;
            atb_y = cfg.axes[1].axis_to_button;
            atb_t = cfg.axes[2].axis_to_button;

            // Apply button debounce (convert ms → consecutive readings; ~500us cycle)
            let debounce_readings = ((cfg.buttons.debounce_ms as u32 * 1000) / 500).max(1) as u8;
            mcp.set_debounce_threshold(debounce_readings);
            debounce_applied = true;

            btn_cfg = cfg.buttons;
        }
        // Apply debounce on first iteration (default config)
        if !debounce_applied {
            mcp.set_debounce_threshold(MCP23S17_DEBOUNCE_COUNT);
            debounce_applied = true;
        }

        let start = embassy_time::Instant::now();

        let rx = sens_x.read().ok();
        let ry = sens_y.read().ok();
        let rt = sens_t.read().ok();
        let raw_btns = match mcp.read() {
            Ok(buttons) => buttons,
            Err(_) => {
                runtime_stats::BUTTON_ERRORS.fetch_add(1, Ordering::Relaxed);
                runtime_stats::BUTTONS_DEGRADED.store(1, Ordering::Relaxed);
                u32::MAX
            }
        };

        // Track per-sensor error counts (deltas from cumulative sensor counters)
        track_delta!(
            runtime_stats::SENSOR_X_ERRORS,
            sens_x.error_count(),
            prev_err_x
        );
        track_delta!(
            runtime_stats::SENSOR_Y_ERRORS,
            sens_y.error_count(),
            prev_err_y
        );
        track_delta!(
            runtime_stats::SENSOR_TWIST_ERRORS,
            sens_t.error_count(),
            prev_err_t
        );

        // V1.23: separate CRC and magnet error counters
        track_delta!(
            runtime_stats::SENSOR_CRC_ERRORS,
            sens_x.crc_error_count(),
            prev_crc_x
        );
        track_delta!(
            runtime_stats::SENSOR_CRC_ERRORS,
            sens_y.crc_error_count(),
            prev_crc_y
        );
        track_delta!(
            runtime_stats::SENSOR_CRC_ERRORS,
            sens_t.crc_error_count(),
            prev_crc_t
        );

        track_delta!(
            runtime_stats::MAGNET_ERRORS,
            sens_x.magnet_error_count(),
            prev_mag_x
        );
        track_delta!(
            runtime_stats::MAGNET_ERRORS,
            sens_y.magnet_error_count(),
            prev_mag_y
        );
        track_delta!(
            runtime_stats::MAGNET_ERRORS,
            sens_t.magnet_error_count(),
            prev_mag_t
        );

        // Update diagnostic atomics (raw sensor values)
        runtime_stats::RAW_AXIS_X
            .store(rx.unwrap_or(MT6826_ANGLE_CENTER) as u32, Ordering::Relaxed);
        runtime_stats::RAW_AXIS_Y
            .store(ry.unwrap_or(MT6826_ANGLE_CENTER) as u32, Ordering::Relaxed);
        runtime_stats::RAW_AXIS_TWIST
            .store(rt.unwrap_or(MT6826_ANGLE_CENTER) as u32, Ordering::Relaxed);

        let out_x = pl_x.process(rx.unwrap_or(MT6826_ANGLE_CENTER), rx.is_some());
        let out_y = pl_y.process(ry.unwrap_or(MT6826_ANGLE_CENTER), ry.is_some());
        let out_t = pl_t.process(rt.unwrap_or(MT6826_ANGLE_CENTER), rt.is_some());

        // Apply button config: invert, then mask
        let mut buttons = raw_btns;
        buttons ^= btn_cfg.inverted_mask;
        buttons &= btn_cfg.enabled_mask;

        // Apply axis-to-button mappings
        apply_axis_to_button(out_x.value, &atb_x, &mut buttons);
        apply_axis_to_button(out_y.value, &atb_y, &mut buttons);
        apply_axis_to_button(out_t.value, &atb_t, &mut buttons);

        // Update diagnostic atomics (processed values + buttons)
        let px = (out_x.value.clamp(-1.0, 1.0) * 32767.0) as i32;
        let py = (out_y.value.clamp(-1.0, 1.0) * 32767.0) as i32;
        let pt = (out_t.value.clamp(-1.0, 1.0) * 32767.0) as i32;
        runtime_stats::PROC_AXIS_X.store(px, Ordering::Relaxed);
        runtime_stats::PROC_AXIS_Y.store(py, Ordering::Relaxed);
        runtime_stats::PROC_AXIS_TWIST.store(pt, Ordering::Relaxed);
        runtime_stats::BUTTON_MASK.store(buttons, Ordering::Relaxed);

        let unhealthy_mask: u8 = if !out_x.healthy { 0x01 } else { 0 }
            | if !out_y.healthy { 0x02 } else { 0 }
            | if !out_t.healthy { 0x04 } else { 0 };
        runtime_stats::SENSOR_UNHEALTHY.store(unhealthy_mask, Ordering::Relaxed);

        REPORT_SIGNAL.signal(GamepadReport {
            x: out_x,
            y: out_y,
            twist: out_t,
            buttons,
        });

        runtime_stats::record_cycle(start.elapsed().as_micros() as u32);
        ticker.next().await;
    }
}
