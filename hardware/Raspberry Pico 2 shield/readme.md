# 🔌 RP2350 HOTAS Expansion Shield

🚧 **Status: Hardware Validation**

Custom expansion board designed for the Raspberry Pi Pico 2 (RP2350) and the OpenHOTAS ecosystem.

The board provides dedicated interfaces for magnetic encoders, GPIO expanders, and future expansion peripherals, simplifying the construction of custom flight simulation controllers while improving wiring reliability and signal integrity.

---

## Overview

The RP2350 HOTAS Expansion Shield serves as the main hardware interface between the RP2350 and external HOTAS components.

Supported applications include:

- Flight sticks
- Throttle quadrants
- Collective controls
- Rudder pedals
- Button panels

The design focuses on modularity, clean routing, and ease of assembly.

---

## Key Features

- Raspberry Pi Pico 2 (RP2350) compatible
- Dual SPI bus architecture
- MT6826S magnetic encoder support
- MCP23S17 GPIO expander support
- JST-based connectors
- Shared interrupt line for button inputs
- Continuous ground plane
- Designed for custom HOTAS hardware

---

## Hardware Architecture

### SPI0

Dedicated to MCP23S17 GPIO expanders for button and switch inputs.

### SPI1

Dedicated to MT6826S magnetic encoders for axis sensing.

This separation keeps sensor traffic isolated from button matrix communication and simplifies firmware scheduling.

---

## Supported Components

### MT6826S

Contactless magnetic encoder used for:

- Pitch
- Roll
- Twist / Rudder

### MCP23S17

SPI GPIO expander used for:

- Grip buttons
- Toggle switches
- Auxiliary controls

---

## Repository Contents

board.png         PCB render
schematic.pdf     Electrical schematic
gerbers/          Manufacturing files
