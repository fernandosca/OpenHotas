#![no_std]
#![no_main]
#![allow(clippy::missing_transmute_annotations)]

mod axis;
mod calibration;
mod config;
mod constants;
mod diagnostics;
mod filters;
mod sensors;
mod spi_bus;
mod storage;
mod tasks;
mod usb;

use embassy_executor::Spawner;
use embassy_rp::flash::Flash;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::otp;
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::class::hid::{Config as HidConfig, HidReaderWriter};
use embassy_usb::{Builder, Config as UsbConfig};
use {defmt_rtt as _, panic_probe as _};

use axis::{AxisConfig, AxisPipeline};
use calibration::data::CalibrationData;
use constants::*;
use diagnostics::runtime_stats;
use sensors::mcp23s::Mcp23s;
use sensors::mt6826::Mt6826;
use storage::flash;
use usb::descriptor::HID_REPORT_DESCRIPTOR;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

use static_cell::StaticCell;

static DD: StaticCell<[u8; 256]> = StaticCell::new();
static CD: StaticCell<[u8; 256]> = StaticCell::new();
static BD: StaticCell<[u8; 256]> = StaticCell::new();
static CB: StaticCell<[u8; 64]> = StaticCell::new();
static HS: StaticCell<embassy_usb::class::hid::State<'static>> = StaticCell::new();
/// CDC state for binary protocol (V1.22).
/// No transmute needed — StaticCell provides &'static mut directly.
static CDC_STATE: StaticCell<embassy_usb::class::cdc_acm::State<'static>> = StaticCell::new();
/// Buffer estático (no_heap) que guarda o serial USB formatado.
/// O Builder do embassy-usb exige `&'static str` para os descritores, então o
/// unique ID da flash é formatado aqui uma única vez no boot.
static SERIAL_STR: StaticCell<[u8; 18]> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // V1.22: pipelines initialized with defaults.
    // cdc_task loads StoredConfigV2 and signals real config via CONFIG_SIGNAL
    // on the first input_task iteration. No dual-load from legacy V1 format.
    let default_cal = CalibrationData::default();
    let pl_x = AxisPipeline::new(default_cal, AxisConfig::default());
    let pl_y = AxisPipeline::new(default_cal, AxisConfig::default());
    let pl_t = AxisPipeline::new(
        default_cal,
        AxisConfig {
            reset_ema_on_dz: true,
            ..Default::default()
        },
    );

    let flash_periph: Flash<
        '_,
        embassy_rp::peripherals::FLASH,
        embassy_rp::flash::Blocking,
        { 2 * 1024 * 1024 },
    > = Flash::new_blocking(p.FLASH);

    // Transmute: Converter lifetime local de Flash para 'static.
    // Sound: periférico vive pela execução inteira, single-core, nunca é dropado.
    flash::init(unsafe { core::mem::transmute(flash_periph) });

    // Serial único por placa: lido de OTP (chip ID 64-bit).
    // O Builder exige &'static str, então o serial é gravado num static (no_heap).
    let serial = chip_id_serial_static();

    let mut spi0_cfg = SpiConfig::default();
    spi0_cfg.frequency = 1_000_000;
    let spi0 = Spi::new_blocking(p.SPI0, p.PIN_6, p.PIN_7, p.PIN_4, spi0_cfg);
    // Spi::new_blocking resets pad pulls, so apply this after construction.
    enable_miso_pullup(PIN_SPI0_MISO);
    // Transmute: Converter lifetime local de Spi para 'static.
    // Sound: SPI0 nunca é dropado, inicialização única, single-core.
    spi_bus::init_spi0(unsafe { core::mem::transmute(spi0) });

    let mut spi1_cfg = SpiConfig::default();
    spi1_cfg.frequency = MT6826_SPI_FREQ_HZ;
    spi1_cfg.polarity = Polarity::IdleHigh;
    spi1_cfg.phase = Phase::CaptureOnSecondTransition;
    let spi1 = Spi::new_blocking(p.SPI1, p.PIN_14, p.PIN_15, p.PIN_12, spi1_cfg);
    enable_miso_pullup(PIN_SPI1_MISO);
    // Transmute: Converter lifetime local de Spi para 'static.
    // Sound: SPI1 nunca é dropado, inicialização única, single-core.
    spi_bus::init_spi1(unsafe { core::mem::transmute(spi1) });

    // Transmute: Converter lifetime local de Mcp23s para 'static.
    // Sound: driver nunca é dropado, inicialização única, single-core.
    let mut mcp23s: Mcp23s<'static> =
        unsafe { core::mem::transmute(Mcp23s::new(Output::new(p.PIN_5, Level::High))) };
    if mcp23s.init().is_err() {
        defmt::warn!("MCP23S17 init failed; continuing with buttons released");
        runtime_stats::BUTTON_ERRORS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        runtime_stats::BUTTONS_DEGRADED.store(1, core::sync::atomic::Ordering::Relaxed);
    }
    // Transmute: Converter lifetime local de Mt6826 para 'static.
    // Sound: drivers nunca são dropados, inicialização única, single-core.
    let sens_x: Mt6826<'static> =
        unsafe { core::mem::transmute(Mt6826::new(Output::new(p.PIN_10, Level::High))) };
    let sens_y: Mt6826<'static> =
        unsafe { core::mem::transmute(Mt6826::new(Output::new(p.PIN_13, Level::High))) };
    let sens_t: Mt6826<'static> =
        unsafe { core::mem::transmute(Mt6826::new(Output::new(p.PIN_16, Level::High))) };

    let driver = Driver::new(p.USB, Irqs);
    let mut usb_cfg = UsbConfig::new(0x16c0, 0x27db);
    usb_cfg.manufacturer = Some("OpenHOTAS");
    usb_cfg.product = Some("OpenHOTAS Gamepad");
    usb_cfg.serial_number = Some(serial);
    usb_cfg.max_power = 100;
    // BCD major.minor derivado do Cargo.toml via build.rs (ex.: 1.3.0 → 0x0130).
    // Mantém o descritor USB sincronizado com a versão SemVer automaticamente.
    usb_cfg.device_release = env!("USB_DEVICE_RELEASE_BCD")
        .parse::<u16>()
        .unwrap_or(0x0100);

    // StaticCell: inicializa no boot, fornece &'static mut sem transmute.
    let hs = HS.init(embassy_usb::class::hid::State::new());
    let dd = DD.init([0u8; 256]);
    let cd = CD.init([0u8; 256]);
    let bd = BD.init([0u8; 256]);
    let cb = CB.init([0u8; 64]);

    let mut builder = Builder::new(driver, usb_cfg, dd, cd, bd, cb);

    let hid_cfg = HidConfig {
        report_descriptor: HID_REPORT_DESCRIPTOR,
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 64,
    };
    let hid: HidReaderWriter<
        '_,
        embassy_rp::usb::Driver<'_, embassy_rp::peripherals::USB>,
        64,
        64,
    > = HidReaderWriter::new(&mut builder, hs, hid_cfg);

    // CDC for binary protocol (V1.22)
    // StaticCell: &'static mut sem transmute.
    let cdc_state = CDC_STATE.init(embassy_usb::class::cdc_acm::State::new());
    let cdc = CdcAcmClass::new(&mut builder, cdc_state, 64);

    let usb = builder.build();
    let (_reader, writer) = hid.split();
    let (cdc_sender, cdc_receiver) = cdc.split();

    spawner.spawn(tasks::hid::usb_task(usb).unwrap());
    spawner.spawn(tasks::hid::hid_task(writer).unwrap());
    spawner
        .spawn(tasks::input::input_task(sens_x, sens_y, sens_t, mcp23s, pl_x, pl_y, pl_t).unwrap());
    spawner.spawn(tasks::diagnostic::diagnostic_task().unwrap());
    spawner.spawn(tasks::cdc::cdc_task(cdc_sender, cdc_receiver).unwrap());
}

/// Keep an unconnected SPI MISO line at a deterministic high level.
///
/// Embassy configures the complete SPI pad and clears pull resistors inside
/// `Spi::new_blocking`, so the pull must be restored afterwards.
fn enable_miso_pullup(pin: u8) {
    embassy_rp::pac::PADS_BANK0.gpio(pin as usize).modify(|w| {
        w.set_pue(true);
        w.set_pde(false);
    });
}



/// Formata um byte (0..255) como dois hex ASCII em `dst[0..2]`.
fn write_hex_byte(dst: &mut [u8], byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    dst[0] = HEX[(byte >> 4) as usize];
    dst[1] = HEX[(byte & 0x0F) as usize];
}

/// Lê o chip ID de OTP (64 bits, rows 0x0-0x3), formata como serial USB
/// único e retorna um `&'static str`.
///
/// Cada placa tem um chip ID distinto gravado em OTP durante fabricação,
/// então dois sticks OpenHOTAS no mesmo host não colidem na enumeração USB.
///
/// Se a OTP estiver ilegível (falha teórica), cai num fallback de serial fixo
/// e emite um `defmt::warn!`. O boot não trava — mas todos os sticks nessa
/// condição passam a compartilhar o mesmo serial, então o configurador deve
/// alertar o usuário ao encontrar o serial de fallback.
///
/// Usa `StaticCell` para alocar o buffer no heap-less firmware — sem `unsafe`
/// além do `from_utf8_unchecked` que é seguro pelo invariante local (ASCII hex).
fn chip_id_serial_static() -> &'static str {
    // Inicializa o buffer via StaticCell (uma vez no boot).
    // Retorna &'static mut [u8; 18] — sem unsafe.
    let buf: &'static mut [u8; 18] = SERIAL_STR.init([0u8; 18]);

    let chip_id = otp::get_chipid().unwrap_or_else(|_| {
        defmt::warn!("OTP chip ID unavailable, using fallback serial");
        0u64
    });

    // Formata o serial diretamente no buffer estático
    buf[0] = b'O';
    buf[1] = b'H';
    for (i, byte) in chip_id.to_be_bytes().iter().enumerate() {
        write_hex_byte(&mut buf[2 + i * 2..4 + i * 2], *byte);
    }

    // Safety: buf contém apenas 'O', 'H' e hex ASCII (0-9, A-F) — UTF-8 válido.
    unsafe { core::str::from_utf8_unchecked(&buf[..18]) }
}
