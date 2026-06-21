#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
pub struct SensorStatus {
    pub mt6826_x_ok: bool,
    pub mt6826_y_ok: bool,
    pub mt6826_twist_ok: bool,
    pub mcp23s_ok: bool,
    pub spi_errors: u32,
}

impl SensorStatus {
    #[allow(dead_code)]
    pub fn all_ok(&self) -> bool {
        self.mt6826_x_ok && self.mt6826_y_ok && self.mt6826_twist_ok && self.mcp23s_ok
    }

    #[allow(dead_code)]
    pub fn add_spi_error(&mut self) {
        self.spi_errors = self.spi_errors.saturating_add(1);
    }

    #[allow(dead_code)]
    pub fn log(&self) {
        defmt::info!(
            "Sensors: X={} Y={} T={} BTN={} errs={}",
            self.mt6826_x_ok,
            self.mt6826_y_ok,
            self.mt6826_twist_ok,
            self.mcp23s_ok,
            self.spi_errors
        );
    }
}
