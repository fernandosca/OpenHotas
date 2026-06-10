# MT6826S Sensor Board

## Overview

This project is a compact breakout board for the MT6826S magnetic encoder, designed to provide a simple and reliable interface for SPI-based applications such as HOTAS systems, joysticks, throttle quadrants, robotics, and other motion-control projects.

The board exposes the essential signals required for integration while maintaining a compact footprint and easy assembly.

## Project Status

🚧 **Work in Progress (WIP)**

This project is currently under development.

The design has been completed and reviewed, but additional hardware testing and validation may still be performed. Future revisions may include improvements, bug fixes, and documentation updates.

## Revision 1.1

### Changes from Revision 1.0

* Removed the PWM output pin from the connector.
* Simplified connector layout and wiring.
* Updated schematic and PCB design.
* Regenerated manufacturing files.

### Why the PWM Pin Was Removed

The intended use of this board relies exclusively on the MT6826S SPI interface. Since the PWM output is not required for the target applications, removing it simplifies the design and connector pinout without affecting normal operation.

## Features

* MT6826S magnetic encoder support
* SPI communication interface
* Compact breakout design
* Easy integration into custom electronics projects
* Suitable for HOTAS, joysticks, throttles, and robotics

## Repository Contents

This repository includes all project files required to review, modify, manufacture, and assemble the board:

* Schematic files
* PCB layout files
* Gerber files
* Bill of Materials (BOM)
* Pick and Place (CPL) files
* Manufacturing outputs
* 3D models and project assets (when available)

## Manufacturing

The board can be manufactured directly from the provided Gerber files.

For PCB assembly services, the included BOM and Pick and Place files can be used to streamline the assembly process.

## License

Open-source hardware project.

## Author

Fernando Scaranello
