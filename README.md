# ✈️ OpenHOTAS

OpenHOTAS is an open-source HOTAS (Hands On Throttle And Stick) platform built around the Raspberry Pi RP2350 and MT6826S magnetic encoders.

The project combines custom electronics, modern Rust firmware, and community-proven mechanical designs to create a modular and fully digital flight control system for simulation enthusiasts.

---

## Features

- RP2350-based controller platform
- Contactless MT6826S magnetic sensors
- USB HID Gamepad support
- Async firmware powered by Embassy
- Modular architecture for future expansion
- Open hardware and open firmware

---

## Hardware Overview

### Microcontroller
- Raspberry Pi RP2350

### Sensors
- MT6826S magnetic encoders

### Input Expansion
- MCP23S17 SPI I/O expanders

### Connectivity
- USB HID

---

## Mechanical Design

OpenHOTAS builds upon excellent community-driven designs:

### Joystick Gimbal
Based on the Olukelo Joystick Gimbal project.

### Grip
Based on the F-16 Sidestick Grip project.

The mechanical designs are adapted and integrated with custom electronics and firmware.

---

## Firmware

The firmware is written in Rust using:

- Embassy
- no_std
- TinyUSB HID stack
- Async task-based architecture

The goal is to provide deterministic input processing, low latency, and maintainable embedded software.

---

## Repository Contents

- Firmware source code
- Hardware schematics
- PCB design files
- Manufacturing outputs
- Documentation
- Assembly guides

---

## Project Status

🚧 Active Development

Current work focuses on:

- Hardware validation
- Firmware integration
- Sensor calibration
- USB HID testing

Future plans include:

- Persistent calibration storage
- Configuration utility
- Dedicated throttle module
- Additional expansion modules

---

## Acknowledgements

Special thanks to the creators of the original community joystick projects that inspired this work.

- Olukelo Joystick Gimbal
- F-16 Sidestick Grip

---

## Contributing

Contributions, testing, bug reports, hardware improvements, and pull requests are always welcome.

---

## License

License information will be added as the project matures.
