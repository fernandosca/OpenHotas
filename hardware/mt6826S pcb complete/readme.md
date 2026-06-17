# 🧲 MT6826S High-Precision Sensor Breakout Board

🚧 **Project Status: Active Development / Hardware Integration (WIP)**

This hardware module is currently in its active testing and validation phase. 

The Revision 1.1 layout has been fully routed, electrically inspected, and verified against high-speed SPI signal constraints. While the electrical design is locked, future minor revisions may introduce subtle mechanical footprint optimizations based on real-world integration with custom 3D-printed gimbal assemblies.

---

## 🎯 Overview

The **MT6826S Sensor Board** is an ultra-compact, high-signal-integrity breakout board engineered specifically for the MagnTek MT6826S 15-bit magnetic encoder. This satellite board functions as the precise angular tracking node for advanced motion-control systems.

Designed as an official hardware component of the **OpenHOTAS** ecosystem, this PCB's physical dimensions and mounting holes are precision-tailored to fit seamlessly into modified STL files of community gimbals, such as the popular **Olukelo Joystick Gimbal**. This eliminates unstable manual wiring and provides a robust, plug-and-play mechanical installation.

The board drops all redundant pins to focus entirely on delivering a clean, low-impedance 4-wire SPI pathway, making it ideal for flight simulator axes, throttle quadrants, direct-drive wheels, and precision robotics.

---

## 💎 Key Features

- **Native MT6826S Support:** Optimized footprint for the core 15-bit contactless magnetic encoder.
- **Ultra-Compact Footprint:** Engineered to be embedded directly inside tight 3D-printed mechanical enclosures.
- **High-Integrity SPI Layout:** Shielded digital traces with tightly coupled ground references to preserve 1MHz+ clock signals.
- **Olukelo Gimbal Ready:** Physical geometry optimized to replace traditional potentiometers without mechanical friction or physical wear.
- **Vibration-Resistant Interconnects:** Clean, standardized pin header layout designed for secure wiring inside moving mechanical assemblies.

---

## 🚀 Revision 1.1 Evolution

### Key Hardware Changes (V1.0 ➔ V1.1):
* **PWM Circuit Eradication:** The legacy analog PWM output pin was completely removed from the connector matrix.
* **Streamlined Routing & Layout:** Truncating the PWM trace allowed for an aggressive simplification of the PCB routing, resulting in an even smaller footprint and enhanced noise shielding for the remaining digital lines.
* **Optimized Manufacturing Outputs:** Completely rebuilt schematic and PCB trace matrix to match production-grade assembly standards.

### Why the PWM Pin Was Removed:
The OpenHOTAS platform and modern high-performance controllers rely exclusively on the MT6826S high-speed digital SPI interface to achieve absolute 15-bit resolution (`0..32767`) and deterministic sub-500µs loops. Removing the unused analog PWM subsystem stripped away unnecessary circuit complexity, cut down trace capacitance, and minimized the physical connector footprint without affecting digital operations.

---

## 📦 Repository Structure & Manufacturing Files

This repository contains full, production-ready industrial manufacturing packages. The outputs are generated to allow automated Turnkey PCB production:

* **Schematics & PCB Layout:** Native design files detailing the component layout and electrical constraints.
* **Production-Grade Gerbers:** High-precision Gerber files (RS-274X/X2) with optimized solder mask and silkscreen layers, ready for instant fabrication (JLCPCB, PCBWay, etc.).
* **Turnkey Assembly Files:** Includes fully mapped **Bill of Materials (BOM)** with precise manufacturer part numbers and exact **Pick and Place (CPL/Centroid)** coordinate files, enabling full automated SMT surface-mount assembly by manufacturing services.
* **Mechanical Data:** 3D Step models (when available) to assist in CAD alignment inside 3D printed joystick grips and gimbal bases.

---

## 🤝 Open Source & Contributions

This board is an open-source hardware project designed to support the DIY flight simulation and robotics community. If you are adapting this board to alternative mechanical gimbals or custom layouts, feedback regarding mechanical tolerances and electrical performance is highly encouraged!

**Author:** Scaranello  
**Part of the Ecosystem:** [OpenHOTAS Project]
