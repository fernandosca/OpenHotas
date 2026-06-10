# RP2350 HOTAS Expansion Shield

🚧 **Project Status: Work in Progress (WIP)**

This project is currently under active development and should be considered experimental.

The current PCB revision has been designed and reviewed, but it has not yet undergone complete hardware assembly, validation, and long-term testing. While every effort has been made to ensure a correct and reliable design, there may still be issues that will only be identified during real-world testing.

Future revisions may include PCB layout improvements, hardware fixes, additional features, and documentation updates based on test results and community feedback.

If you decide to manufacture or use this board, please do so with the understanding that the design is still being validated and may change in future releases.

---

## Overview

The RP2350 HOTAS Expansion Shield is a compact expansion board designed to simplify the development of custom flight simulation controllers, including joysticks, throttles, rudder pedals, button boxes, and complete HOTAS (Hands On Throttle And Stick) systems.

The shield is intended to work with RP2350-based boards and provides convenient interfaces for magnetic encoders, GPIO expanders, buttons, switches, and other peripherals commonly found in simulator controls.

The primary goal of this project is to provide a flexible, easy-to-wire, and expandable platform for DIY simulator enthusiasts while maintaining a clean PCB layout and reliable signal routing.

---

## Features

- Compatible with RP2350-based boards
- Shared SPI bus architecture
- Support for MT6826S magnetic encoders
- Support for MCP23S17 SPI GPIO expanders
- Expansion headers for additional peripherals
- Dedicated power and signal connections
- Ground plane optimized for signal integrity
- Compact and modular design
- Suitable for HOTAS and custom controller projects

---

## Supported Devices

### Magnetic Encoders

- MT6826S

The MT6826S provides high-resolution, contactless position sensing using magnetic technology, making it ideal for joystick gimbals, throttle levers, and rudder pedal mechanisms.

### GPIO Expansion

- MCP23S17

The MCP23S17 allows the addition of 16 extra digital inputs or outputs through SPI communication, making it easy to support large numbers of buttons, switches, and auxiliary controls.

---

## Applications

This shield can be used in a wide variety of simulator and control projects, including:

- Flight simulator joysticks
- HOTAS systems
- Collective controls
- Throttle quadrants
- Rudder pedals
- Button boxes
- Industrial control panels
- Custom gaming controllers

---

## SPI Architecture

The board uses a shared SPI communication bus that allows multiple devices to operate simultaneously.

Supported SPI peripherals include:

- MT6826S magnetic encoders
- MCP23S17 GPIO expanders

Each peripheral uses its own dedicated Chip Select (CS) signal, allowing efficient communication while minimizing wiring complexity.

---

## Design Goals

This project was developed with the following objectives:

- Simplify wiring and assembly
- Provide reliable SPI communication
- Support high button-count controller designs
- Allow easy future expansion
- Maintain a compact PCB footprint
- Be accessible to DIY builders and makers

