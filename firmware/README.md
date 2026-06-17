# 🦀 OpenHOTAS Firmware

Rust firmware for the OpenHOTAS platform.

The firmware targets RP2350-based boards and uses Embassy to provide a modular, asynchronous foundation for custom flight simulation controls.

---

## Overview

The firmware is responsible for:

* Reading magnetic position sensors
* Processing button and switch inputs
* Managing USB HID communication
* Applying calibration and filtering logic
* Supporting future configuration and expansion features

---

## Technology Stack

* Rust
* Embassy
* no_std
* RP2350
* USB HID
* defmt logging

---

## Target Platform

This firmware targets RP2350A-based boards, including:

* Raspberry Pi Pico 2
* Compatible RP2350A development boards

The default build target is configured in `.cargo/config.toml`:

```toml
[build]
target = "thumbv8m.main-none-eabihf"
```

---

## Hardware Compatibility

This firmware is primarily developed for the custom OpenHOTAS hardware platform.

Default pin assignments, peripheral mappings, and hardware configurations are designed around the OpenHOTAS PCBs found in the `hardware/` directory.

When adapting the firmware to other RP2350-based boards, developers should review and adjust:

* GPIO assignments
* SPI bus configuration
* Chip Select mappings
* Interrupt pins
* Peripheral connections

Refer to the hardware documentation for the reference implementation.

---

## Requirements

Install the Rust target:

```bash
rustup target add thumbv8m.main-none-eabihf
```

Optional tools:

```bash
cargo install elf2uf2-rs
cargo install probe-rs-tools
```

---

## Building

Build the firmware:

```bash
cargo build --release
```

Generated binaries are located in:

```text
target/thumbv8m.main-none-eabihf/release/
```

---

## UF2 Conversion

To generate a UF2 file for USB bootloader flashing:

```bash
elf2uf2-rs \
  target/thumbv8m.main-none-eabihf/release/openhotas \
  openhotas.uf2
```

---

## Running with probe-rs

The project runner is configured in `.cargo/config.toml`:

```toml
[target.thumbv8m.main-none-eabihf]
runner = "probe-rs run --chip RP2350"
```

Run the firmware with:

```bash
cargo run --release
```

---

## Directory Structure

```text
src/
├── constants/     Shared constants and configuration values
├── driver/        Hardware drivers
├── flash/         Persistent storage support
├── hid/           USB HID implementation
├── pipeline/      Input processing pipeline
├── task/          Embassy tasks
├── types/         Shared data structures
└── main.rs        Application entry point
```

---

## Current Features

* MT6826S magnetic encoder support
* MCP23S17 GPIO expander support
* USB HID gamepad interface
* Asynchronous task architecture
* Multi-axis input processing

---

## Development Status

🚧 Active Development

Current focus:

* Hardware validation
* Firmware integration
* Sensor calibration
* USB HID testing

Planned features:

* Persistent calibration storage
* Runtime configuration support
* Diagnostic tools
* Additional expansion modules

---

## OpenHOTAS

This firmware is part of the OpenHOTAS ecosystem and works alongside the custom hardware modules found in this repository.
