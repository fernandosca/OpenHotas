pub mod mcp23s;
pub mod mt6826;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum SensorError {
    SpiError,
    CrcError,
    MagnetError,
    Timeout,
    NotInitialized,
}

pub trait Sensor {
    type Output;

    fn read(&mut self) -> Result<Self::Output, SensorError>;

    #[allow(dead_code)]
    fn is_healthy(&self) -> bool;

    #[allow(dead_code)]
    fn error_count(&self) -> u32;
}
