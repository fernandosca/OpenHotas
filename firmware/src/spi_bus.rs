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

pub static SPI0_BUS: Mutex<CriticalSectionRawMutex, RefCell<Option<Spi0Bus>>> =
    Mutex::new(RefCell::new(None));
pub static SPI1_BUS: Mutex<CriticalSectionRawMutex, RefCell<Option<Spi1Bus>>> =
    Mutex::new(RefCell::new(None));

pub fn init_spi0(spi: Spi0Bus) {
    SPI0_BUS.lock(|state| {
        *state.borrow_mut() = Some(spi);
    });
}

pub fn init_spi1(spi: Spi1Bus) {
    SPI1_BUS.lock(|state| {
        *state.borrow_mut() = Some(spi);
    });
}

pub fn with_spi0<R>(f: impl FnOnce(&mut Spi0Bus) -> R) -> Result<R, SpiBusError> {
    SPI0_BUS.lock(|state| {
        let mut bus_ref = state.borrow_mut();
        let bus = bus_ref.as_mut().ok_or(SpiBusError::NotInitialized)?;
        Ok(f(bus))
    })
}

pub fn with_spi1<R>(f: impl FnOnce(&mut Spi1Bus) -> R) -> Result<R, SpiBusError> {
    SPI1_BUS.lock(|state| {
        let mut bus_ref = state.borrow_mut();
        let bus = bus_ref.as_mut().ok_or(SpiBusError::NotInitialized)?;
        Ok(f(bus))
    })
}
