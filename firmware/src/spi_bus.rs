//! Acesso global single-core ao SPI0 (MCP23S17) e SPI1 (MT6826S).
//!
//! # Arquitetura
//!
//! `Mutex<RefCell<Option<Spi>>>` — não `static mut` nem `unsafe` para acesso concorrente.
//! O `CriticalSectionRawMutex` é suficiente porque o RP2350 roda single-core
//! (embassy executa num core só; o segundo core está desligado). O Mutex garante
//! que dentro de `with_spi*` ninguém mais toma emprestado o Spi, prevenindo
//! reentrância mesmo que o executor misture tasks.
//!
//! `RefCell` em vez de `Cell`: `Spi` não é `Copy` nem `Send`, então `borrow_mut()`
//! é a única maneira de obter `&mut Spi` de dentro do lock.
//!
//! # Modo de falha
//!
//! Se `init_spi*` não for chamado antes do primeiro `with_spi*`, retorna
//! `SpiBusError::NotInitialized` — o caller (sensor driver) deve propagar como
//! `SensorError::NotInitialized`. Atualmente NENHUM caller trata esse erro
//! além de retornar Err; o input_task interpreta como sensor ausente.
//! Isso é um gap: se a inicialização do SPI falhar silenciosamente (ex: PIO
//! block errado), todos os sensores reportam erro e o firmware roda sem
//! leituras — sem reboot, sem alarme no HID.
//!
//! # no_std / heap
//!
//! Sem heap. Mutex é blocking (não async) — adequado para SPI blocking.
//! As transações SPI ocorrem dentro da critical section, bloqueando o
//! scheduler do embassy por alguns microssegundos.

use core::cell::RefCell;

use embassy_rp::peripherals::{SPI0, SPI1};
use embassy_rp::spi::{Blocking, Spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;

pub type Spi0Bus = Spi<'static, SPI0, Blocking>;
pub type Spi1Bus = Spi<'static, SPI1, Blocking>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiBusError {
    NotInitialized,
}

// Mutex global por barramento SPI.
// Inicializado como None; init_spi* define o valor uma vez no boot.
// O Mutex garante que as transações SPI sejam serializadas — não pode
// haver duas transações simultâneas no mesmo periférico SPI.
pub static SPI0_BUS: Mutex<CriticalSectionRawMutex, RefCell<Option<Spi0Bus>>> =
    Mutex::new(RefCell::new(None));
pub static SPI1_BUS: Mutex<CriticalSectionRawMutex, RefCell<Option<Spi1Bus>>> =
    Mutex::new(RefCell::new(None));

/// Inicializa o barramento SPI0 (MCP23S17).
/// Deve ser chamado uma vez no boot, ANTES de qualquer `with_spi0`.
/// Se não for chamado, `with_spi0` retorna `NotInitialized`.
pub fn init_spi0(spi: Spi0Bus) {
    SPI0_BUS.lock(|state| {
        *state.borrow_mut() = Some(spi);
    });
}

/// Inicializa o barramento SPI1 (MT6826S).
/// Mesmas regras de `init_spi0`.
pub fn init_spi1(spi: Spi1Bus) {
    SPI1_BUS.lock(|state| {
        *state.borrow_mut() = Some(spi);
    });
}

/// Executa `f` com acesso `&mut` ao SPI0.
/// Retorna `Err(NotInitialized)` se o barramento não foi inicializado.
/// Se `f` falhar (ex: timeout SPI), o erro é propagado para o caller
/// (MCP23S17 driver), que incrementa `error_count`.
/// ATUALMENTE nenhum timeout é implementado — a HAL do embassy_rp para
/// SPI blocking não tem timeout configurável; uma linha MISO presa em
/// low pode travar o barramento indefinidamente dentro de `f`.
pub fn with_spi0<R>(f: impl FnOnce(&mut Spi0Bus) -> R) -> Result<R, SpiBusError> {
    SPI0_BUS.lock(|state| {
        let mut bus_ref = state.borrow_mut();
        let bus = bus_ref.as_mut().ok_or(SpiBusError::NotInitialized)?;
        Ok(f(bus))
    })
}

/// Executa `f` com acesso `&mut` ao SPI1.
/// Mesmas regras de `with_spi0`, incluindo a ausência de timeout.
pub fn with_spi1<R>(f: impl FnOnce(&mut Spi1Bus) -> R) -> Result<R, SpiBusError> {
    SPI1_BUS.lock(|state| {
        let mut bus_ref = state.borrow_mut();
        let bus = bus_ref.as_mut().ok_or(SpiBusError::NotInitialized)?;
        Ok(f(bus))
    })
}
