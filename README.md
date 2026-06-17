# ✈️ OpenHOTAS

OpenHOTAS is an open-source HOTAS (Hands On Throttle And Stick) platform built around the Raspberry Pi RP2350 and contactless magnetic sensors.

The goal is to provide a modern, modular, and fully digital flight control platform combining custom hardware, Rust firmware, and community-inspired mechanical designs.

---

## Features

- Raspberry Pi RP2350 based controller
- Contactless magnetic sensing
- USB HID gamepad support
- Embassy async firmware architecture
- Modular hardware design
- Open-source hardware and firmware
- Designed for low-latency flight simulation controls

---

## Inspiration

OpenHOTAS is inspired by two excellent community projects:

- Olukelo Joystick Gimbal
- F-16 Sidestick Grip

These proven mechanical concepts serve as the foundation for a platform built around custom electronics, modern firmware, and a fully digital architecture.

---

## Technology Stack

### Hardware

- Raspberry Pi RP2350
- MT6826S magnetic encoders
- MCP23S17 SPI I/O expanders
- Custom PCBs
- USB connectivity

### Firmware

- Rust
- Embassy
- no_std
- TinyUSB HID
- Async task-based architecture

---

## Repository Structure

```text
firmware/    Rust firmware source code
hardware/    Schematics, PCB files and manufacturing data
dev/         Development resources, AI context files and project references
```

---

## Project Status

🚧 Active Development

Current focus:

- Hardware validation
- Firmware integration
- Sensor calibration
- USB HID testing

Planned features:

- Persistent calibration storage
- Configuration utility
- Dedicated throttle module
- Expansion modules

---

## Contributing

Contributions, testing, bug reports, hardware improvements, and pull requests are welcome.

---

## License

License information will be added in a future release.
