//! Traits e tipos compartilhados para sensores (MT6826S e MCP23S17).

pub mod mcp23s;
pub mod mt6826;

/// Erros de sensor padronizados para todos os periféricos.
///
/// - `SpiError`: falha de comunicação SPI (timeout, frame error).
/// - `CrcError`: CRC do payload não confere (MT6826S).
/// - `MagnetError`: campo magnético insuficiente ou subtensão (MT6826S).
/// - `NotPresent`: sensor não respondeu (MISO 0xFF detection no MT6826S;
///   readback verification no MCP23S17).
/// - `NotInitialized`: barramento SPI ou driver não inicializado.
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum SensorError {
    SpiError,
    CrcError,
    MagnetError,
    NotPresent,
    NotInitialized,
}

/// Trait comum para leitura de sensores.
///
/// `read()` retorna o valor lido ou um `SensorError`.
/// `error_count()` retorna o total acumulado de erros (útil para diagnóstico).
pub trait Sensor {
    type Output;

    fn read(&mut self) -> Result<Self::Output, SensorError>;

    fn error_count(&self) -> u32;
}
