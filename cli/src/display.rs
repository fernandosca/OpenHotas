//! Display formatting — converts protocol types into human-readable strings.

use openhotas_protocol::config::{AxisConfig, DeviceConfig};
use openhotas_protocol::diagnostics::{
    ButtonStates, ErrorCounters, ProcessedAxes, RawAxes, RuntimeStats, SensorStatusReport,
};
use openhotas_protocol::request::AxisId;
use openhotas_protocol::response::DeviceInfo;

/// Human-readable axis name.
pub fn axis_name(axis: AxisId) -> &'static str {
    match axis {
        AxisId::X => "X",
        AxisId::Y => "Y",
        AxisId::Twist => "Twist",
    }
}

// ── DeviceInfo ───────────────────────────────────────────────────────────

pub fn format_info(info: &DeviceInfo) -> String {
    let fw = std::str::from_utf8(&info.firmware_version)
        .unwrap_or("?")
        .trim_matches('\0');
    let git = std::str::from_utf8(&info.git_hash)
        .unwrap_or("?")
        .trim_matches('\0');

    format!(
        "OpenHOTAS Firmware\n\
         ─────────────────\n\
         Version:     {fw}\n\
         Git Hash:    {git}\n\
         Protocol:    v{}.{}\n\
         Axes:        {}\n\
         Buttons:     {}",
        info.protocol_major, info.protocol_minor, info.axis_count, info.button_count,
    )
}

// ── DeviceConfig ─────────────────────────────────────────────────────────

pub fn format_config(cfg: &DeviceConfig) -> String {
    let mut out = String::from("Active Configuration\n──────────────────\n");

    for (i, ax) in cfg.axes.iter().enumerate() {
        let name = axis_name(AxisId::from_usize(i));
        out.push_str(&format!(
            "\n[{name}] enabled={} invert={} deadzone={}‰ ema={}‰\n",
            ax.enabled, ax.inverted, ax.deadzone_permille, ax.ema_permille,
        ));
        out.push_str(&format!(
            "  curve: P1=({},{}) P3=({},{})\n",
            ax.response_curve.point_left.x,
            ax.response_curve.point_left.y,
            ax.response_curve.point_right.x,
            ax.response_curve.point_right.y,
        ));
        out.push_str(&format!(
            "  center_offset: {}‰\n",
            ax.center_offset_permille,
        ));
        if ax.axis_to_button.enabled {
            out.push_str(&format!(
                "  axis_to_button: threshold={}‰ dir={:?} button={}\n",
                ax.axis_to_button.threshold_permille,
                ax.axis_to_button.direction,
                ax.axis_to_button.button_index,
            ));
        }
        out.push_str(&format!(
            "  cal: min={} center={} max={}\n",
            ax.calibration.min_raw, ax.calibration.center_raw, ax.calibration.max_raw,
        ));
        out.push_str(&format!(
            "  travel_limit: ±{}%  max_jump={}  reset_ema_on_dz={}\n",
            ax.travel.travel_limit_pct, ax.max_jump_raw, ax.reset_ema_on_dz,
        ));
    }

    out.push_str(&format!(
        "\n[Buttons] enabled={:#010X} inverted={:#010X} debounce={}ms",
        cfg.buttons.enabled_mask, cfg.buttons.inverted_mask, cfg.buttons.debounce_ms,
    ));

    out
}

/// Format a single axis config (used after set-axis).
pub fn format_axis_config(axis: AxisId, ax: &AxisConfig) -> String {
    let name = axis_name(axis);
    format!(
        "[{name}] enabled={} invert={}\n\
         deadzone={}‰  ema={}‰  max_jump={}\n\
         curve: P1=({},{}) P3=({},{})\n\
         cal: min={} center={} max={}\n\
         travel_limit: ±{}%",
        ax.enabled,
        ax.inverted,
        ax.deadzone_permille,
        ax.ema_permille,
        ax.max_jump_raw,
        ax.response_curve.point_left.x,
        ax.response_curve.point_left.y,
        ax.response_curve.point_right.x,
        ax.response_curve.point_right.y,
        ax.calibration.min_raw,
        ax.calibration.center_raw,
        ax.calibration.max_raw,
        ax.travel.travel_limit_pct,
    )
}

// ── Diagnostics ──────────────────────────────────────────────────────────

pub fn format_raw_axes(raw: &RawAxes) -> String {
    format!(
        "Raw Axes (u16, 0–32767)\n\
         ──────────────────────\n\
         X:     {}\n\
         Y:     {}\n\
         Twist: {}",
        raw.x, raw.y, raw.twist,
    )
}

pub fn format_processed_axes(p: &ProcessedAxes) -> String {
    let x_pct = p.x as f32 / 327.67;
    let y_pct = p.y as f32 / 327.67;
    let t_pct = p.twist as f32 / 327.67;
    format!(
        "Processed Axes (i16, -32767..+32767)\n\
         ──────────────────────────────────\n\
         X:     {x:6} ({x_pct:+.1}%)\n\
         Y:     {y:6} ({y_pct:+.1}%)\n\
         Twist: {t:6} ({t_pct:+.1}%)\n\
         Health: X={hx} Y={hy} Twist={ht}",
        x = p.x,
        y = p.y,
        t = p.twist,
        x_pct = x_pct,
        y_pct = y_pct,
        t_pct = t_pct,
        hx = if p.unhealthy_mask & 0x01 == 0 {
            "OK"
        } else {
            "ERR"
        },
        hy = if p.unhealthy_mask & 0x02 == 0 {
            "OK"
        } else {
            "ERR"
        },
        ht = if p.unhealthy_mask & 0x04 == 0 {
            "OK"
        } else {
            "ERR"
        },
    )
}

pub fn format_buttons(b: &ButtonStates) -> String {
    format!(
        "Button States (32 buttons)\n\
         ────────────────────────\n\
         Mask: {:#010X}\n\
         Active: {}",
        b.mask,
        (0..32)
            .filter(|i| (b.mask >> i) & 1 != 0)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    )
}

pub fn format_stats(s: &RuntimeStats) -> String {
    format!(
        "Runtime Stats (since boot)\n\
         ─────────────────────────\n\
         Reports sent:  {}\n\
         Send errors:   {}\n\
         Sensor cycles: {}\n\
         Last cycle:    {} µs\n\
         Max cycle:     {} µs",
        s.reports_sent, s.send_errors, s.sensor_cycles, s.last_cycle_us, s.max_cycle_us,
    )
}

pub fn format_sensor_status(s: &SensorStatusReport) -> String {
    fn status(h: bool, errs: u32) -> String {
        if h {
            format!("OK ({errs} errors)")
        } else {
            format!("UNHEALTHY ({errs} errors)")
        }
    }
    format!(
        "Sensor Status\n\
         ─────────────\n\
         X:     {}\n\
         Y:     {}\n\
         Twist: {}",
        status(s.x.healthy, s.x.error_count),
        status(s.y.healthy, s.y.error_count),
        status(s.twist.healthy, s.twist.error_count),
    )
}

pub fn format_errors(e: &ErrorCounters) -> String {
    let total = e.protocol_crc_errors
        + e.sensor_crc_errors
        + e.magnet_errors
        + e.flash_errors
        + e.button_errors;
    format!(
        "Error Counters (since boot)\n\
         ──────────────────────────\n\
         Protocol CRC:  {}\n\
         Sensor CRC:    {}\n\
         Magnet:        {}\n\
         Flash:         {}\n\
         Buttons:       {} ({})\n\
         ──────────────────────────\n\
         Total errors:  {}",
        e.protocol_crc_errors,
        e.sensor_crc_errors,
        e.magnet_errors,
        e.flash_errors,
        e.button_errors,
        if e.buttons_degraded { "DEGRADED" } else { "OK" },
        total,
    )
}
