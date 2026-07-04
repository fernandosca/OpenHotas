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

/// Estado de saúde padronizado para todos os sensores.
///
/// Centraliza a distinção entre falha de sensor individual e falha de
/// barramento (que afeta TODOS os sensores no mesmo SPI).
///
/// - `Healthy`: última leitura bem-sucedida, sem erros.
/// - `Degraded`: erro ao nível do sensor (CRC, magnet, ausente). O eixo
///   pode continuar operando com fallback, mas a qualidade está reduzida.
/// - `Failed`: erro ao nível do barramento (SPI não inicializado, MISO
///   preso). NENHUM sensor no barramento responde. Requer reboot ou
///   intervenção.
///
/// A transição entre estados é monotônica em direção à degradação:
/// `Healthy → Degraded → Failed`. Recuperação de `Degraded` para `Healthy`
/// é possível (ex: CRC falha num ciclo e passa no seguinte).
/// `Failed` não se recupera sem reboot (o barramento não se repara sozinho).
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum SensorHealth {
    Healthy,
    Degraded,
    Failed,
}

/// Trait comum para leitura de sensores.
///
/// `read()` retorna o valor lido ou um `SensorError`.
/// `error_count()` retorna o total acumulado de erros (útil para diagnóstico).
/// `health()` retorna o estado de saúde atual do sensor, distinguindo entre
/// erro de sensor individual (`Degraded`) e erro de barramento (`Failed`).
pub trait Sensor {
    type Output;

    fn read(&mut self) -> Result<Self::Output, SensorError>;

    fn error_count(&self) -> u32;

    /// Estado de saúde atual do sensor.
    ///
    /// Default: `Healthy`. Drivers devem sobrescrever para reportar
    /// degradação real detectada durante `read()`.
    fn health(&self) -> SensorHealth {
        SensorHealth::Healthy
    }
}
