# 🔌 RP2350 HOTAS Expansion Shield

🚧 **Project Status: Active Development / Hardware Integration (WIP)**

This project is currently under active development and hardware validation. 

The current PCB revision has been fully routed, inspected, and generated. However, it is undergoing final bench assembly, signal integrity checking, and long-term stress testing. While the electrical design adheres strictly to high-speed digital routing principles, future revisions may introduce optimization tweaks, layout layout improvements, and feature expansions based on physical hardware test results.

If you choose to manufacture or deploy this board in its current state, please do so understanding that the layout is in its live validation phase.

---

## 🎯 Overview

The **RP2350 HOTAS Expansion Shield** is a high-performance, low-latency daughterboard meticulously designed to simplify the physical assembly of custom flight simulation hardware. It acts as the central nervous system for advanced Joysticks, Throttle Quadrants, Collective Controls, and complete HOTAS ecosystems.

Engineered specifically to host the **Raspberry Pi Pico 2 (RP2350)**, the shield breaks out raw silicon pins into robust, noise-shielded, and keyed interconnects. It provides dedicated, hardware-isolated pathways for magnetic position encoders, high-speed input expanders, and auxiliary analog/digital peripherals.

The core objective of this design is to eliminate messy point-to-point point prototyping wires, drastically reduce electromagnetic interference (EMI), and deliver a clean, industrial-grade routing solution for DIY flight simulation builders.

---

## 💎 Key Features

- **RP2350 Native Integration:** Tailored footprint for the Raspberry Pi Pico 2 form factor.
- **Dual-Bus Isolated SPI Architecture:** Separate physical routings for SPI0 and SPI1 to eliminate clock crosstalk and bus contention.
- **Keyed JST Interconnects:** All sensor and expansion lines utilize secure, vibration-resistant JST connectors, ensuring perfect electrical contacts for high-frequency lines.
- **High-Resolution Magnetic Tracking:** Native multi-device support for MT6826S contactless absolute encoders.
- **Massive I/O Expansion:** Dedicated slots for MCP23S17 expanders, pushing matrix support up to 32+ low-latency discrete inputs.
- **Wired-OR Interrupt Topology:** Shared open-drain interrupt line with active hardware pull-up, routing asynchronous button events directly to a single GPIO.
- **EMI-Optimized Plane:** Continuous Low-Impedance Ground Plane (`GND`) to maximize signal integrity and shield high-speed SPI signals from environmental noise.

---

## 🧬 Supported Silicon & Core Components

### 🔄 MT6826S Magnetic Encoders
The shield provides dedicated lines for the **MT6826S** 15-bit contactless magnetic encoder. By delivering stable 3.3V power routing and tightly coupled SPI lanes, the board guarantees the electrical stability required to stream high-precision angular data (`0` to `32767` absolute points) across Pitch, Roll, and Twist (Rudder) axes without signal degradation.

### 🎛️ MCP23S17 SPI GPIO Expanders
To support high-button-count flight grips (like the F-16 or A-10 sticks), the shield integrates dual **MCP23S17** expanders over SPI. It configures hardware-addressable opcodes via physical pin strapping (`HAEN`), allowing multiple expanders to safely share a single chip-select line while exposing a complete 32-button input matrix.

---

## ⚡ Electrical & SPI Bus Architecture

To achieve the deterministic sub-500µs loop times required by advanced async firmware (like the Embassy RTOS framework), the shield enforces strict **Dual-Bus Isolation**:

| Bus Interface | Target Peripherals | Physical Implementation | Routing Priority |
| :--- | :--- | :--- | :--- |
| **SPI0** | 2× MCP23S17 Expanders | Matrix Grip Buttons & Toggles | Low-impedance parallel lines |
| **SPI1** | 3× MT6826S Encoders | High-Speed Axis Tracking (X, Y, Twist) | Dedicated manual CS traces |

* **Manual Chip-Select Control:** Hardware multiplexing bottlenecks are bypassed by routing independent, software-controlled GPIO lines for each encoder's Chip Select (`CS`), ensuring atomic SPI transfers.
* **Open-Drain Interrupts:** The I/O expander interrupt pins (`INTA`/`INTB`) are unified into a single hardware-wired line back to the RP2350, utilizing the shield's on-board pull-up framework to handle instant asynchronous debouncing.

---

## 🛠️ Applications & Use Cases

This expansion shield provides a universal, scalable platform suitable for:
- Professional-grade Flight Simulator Joysticks (Force Sensing or Cam-Gimbal based).
- Complex Throttle Quadrants with detents and auxiliary axes.
- Helicopter Collective controls with integrated multi-switch grips.
- Ergonomic Rudder Pedals with differential braking.
- Dedicated Cockpit Button Boxes and Industrial Control Panels.

---

## 🤝 Repository & Manufacturing Goals

The RP2350 HOTAS Expansion Shield is a core component of the OpenHOTAS open-source ecosystem. This section of the repository houses all production-ready manufacturing resources, including:
- Production **Gerber Files** (ready for JLCPCB, PCBWay, etc.).
- Complete **Bill of Materials (BOM)** with exact part numbers for components and JST headers.
- **Component Placement Lists (CPL)** for optional automated SMT assembly.
- Comprehensive hardware assembly and wiring guides.

Community modifications, alternate connector layouts, and testing feedback regarding signal integrity are highly encouraged.
