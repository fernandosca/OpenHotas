pub mod mcp23s;
pub mod mt6826;

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum SensorError {
    SpiError,
    CrcError,
    MagnetError,
    NotInitialized,
}

pub trait Sensor {
    type Output;

    fn read(&mut self) -> Result<Self::Output, SensorError>;

    fn error_count(&self) -> u32;
}
