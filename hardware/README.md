# 🔧 OpenHOTAS Hardware

This directory contains the hardware designs that make up the OpenHOTAS ecosystem.

The hardware platform is built around the Raspberry Pi RP2350 and magnetic position sensing, providing a modular foundation for custom flight simulation controls such as joysticks, throttles, rudder pedals, and control panels.

---

## Contents

### RP2350 HOTAS Expansion Shield

Main controller board based on the Raspberry Pi Pico 2 (RP2350).

Features include:

* Dual SPI architecture
* MT6826S sensor support
* MCP23S17 GPIO expander support
* USB HID integration
* Modular peripheral connectivity

### MT6826S Sensor Board

Compact breakout board designed for the MT6826S magnetic encoder.

Features include:

* Contactless position sensing
* SPI communication
* Compact form factor
* Integration with custom gimbal assemblies

---

## Manufacturing Files

Individual hardware directories may contain:

* Schematics
* PCB design files
* Gerber files
* Bill of Materials (BOM)
* Pick and Place (CPL) files
* Assembly references

Refer to the README inside each hardware module for detailed information.

---

## Project Status

🚧 Active Development

Hardware designs are being validated alongside firmware development and may continue to evolve as testing progresses.

---

## OpenHOTAS

OpenHOTAS is an open-source project focused on creating modular, high-performance flight simulation controls using modern hardware and Rust-based firmware.
