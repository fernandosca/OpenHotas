use crate::axis::AxisPipeline;
use crate::constants::MT6826_ANGLE_CENTER;
use crate::diagnostics::runtime_stats;
use crate::sensors::mcp23s::Mcp23s;
use crate::sensors::mt6826::Mt6826;
use crate::sensors::Sensor;
use crate::usb::hid_gamepad::{GamepadReport, REPORT_SIGNAL};

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
    loop {
        let start = embassy_time::Instant::now();

        let rx = sens_x.read().ok();
        let ry = sens_y.read().ok();
        let rt = sens_t.read().ok();
        let btns = mcp.read().unwrap_or(0);

        let out_x = pl_x.process(rx.unwrap_or(MT6826_ANGLE_CENTER), rx.is_some());
        let out_y = pl_y.process(ry.unwrap_or(MT6826_ANGLE_CENTER), ry.is_some());
        let out_t = pl_t.process(rt.unwrap_or(MT6826_ANGLE_CENTER), rt.is_some());

        REPORT_SIGNAL.signal(GamepadReport {
            x: out_x,
            y: out_y,
            twist: out_t,
            buttons: btns,
        });

        runtime_stats::record_cycle(start.elapsed().as_micros() as u32);
    }
}
