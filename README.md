✈️ OpenHotas

Modular open-source HOTAS platform featuring the RP2350 microcontroller, MT6826S high-precision magnetic sensors, custom PCB design, and USB HID support.

This project is based on well-known community flight controller designs such as the 
Olukelo joystick gimbal design
https://www.printables.com/model/565234-olukelo-joystick-gimbal-threaded-arm-and-sensor-mo

and the 

F-16 sidestick grip design
https://www.printables.com/model/233472-f-16-sidestick-grip
adapting and extending them into a modern, fully digital and modular system.

The main upgrade over traditional designs is the use of MT6826S magnetic encoders, replacing mechanical potentiometers to provide higher precision, better durability, and zero physical wear over time.

The system is built around the Raspberry Pi RP2350, enabling native USB HID output, low-latency input processing, and flexible firmware development for simulation and custom control mapping.

In addition to pitch and roll axes, the design also includes a twist axis (rudder control), providing a complete 3-axis input solution for flight simulation without the need for external pedals.

🚧 Project Status

This project is currently under active development. Hardware layouts,# ✈️ OpenHOTAS

**OpenHOTAS** is a modular open-source HOTAS (Hands On Throttle And Stick) platform built around the Raspberry Pi RP2350 microcontroller and MT6826S high-precision magnetic encoders.

The goal of this project is to provide a modern, fully digital, and expandable flight control system for flight simulation enthusiasts, combining custom electronics, open-source hardware, and community-driven mechanical designs.

## Inspiration and Acknowledgements

This project builds upon and extends several outstanding community designs:

* Olukelo Joystick Gimbal Design
  https://www.printables.com/model/565234-olukelo-joystick-gimbal-threaded-arm-and-sensor-mo

* F-16 Sidestick Grip Design
  https://www.printables.com/model/233472-f-16-sidestick-grip

OpenHOTAS adapts these proven mechanical concepts into a modern electronic platform with custom PCBs, magnetic sensing technology, and native USB HID support.

## Key Features

* Raspberry Pi RP2350-based controller architecture
* MT6826S high-resolution magnetic encoders
* Native USB HID support
* Custom-designed PCBs
* Modular and expandable hardware architecture
* Open-source firmware and hardware
* Designed for DIY flight simulation projects

## Magnetic Encoder Upgrade

Traditional joystick designs often rely on potentiometers, which suffer from mechanical wear, signal degradation, and limited lifespan.

OpenHOTAS replaces these components with MT6826S contactless magnetic encoders, providing:

* Higher positional accuracy
* Improved repeatability
* Smooth operation
* Zero mechanical wear
* Long-term reliability

## 3-Axis Control System

In addition to the primary pitch and roll axes, OpenHOTAS incorporates a dedicated twist axis for rudder control.

This allows a complete 3-axis flight control solution without requiring external rudder pedals, making it suitable for compact simulator setups while maintaining precise control.

## System Architecture

The platform is built around the Raspberry Pi RP2350, enabling:

* Native USB HID device support
* Low-latency input processing
* Flexible firmware development
* Future expansion through SPI and GPIO peripherals
* Support for additional sensors, buttons, and control modules

## Project Status

🚧 **Work in Progress (WIP)**

This project is currently under active development.

Hardware, firmware, and mechanical components are continuously evolving as new features are implemented and tested.

Future revisions may include improvements to:

* PCB designs and hardware layouts
* Firmware architecture
* Mechanical assemblies and mounting solutions
* Twist (rudder) mechanism
* Throttle modules
* Button and switch expansion
* Additional control peripherals

As development progresses, design files, documentation, firmware, and manufacturing resources will be updated within this repository.

## Repository Goals

OpenHOTAS aims to become a complete open-source ecosystem for custom flight simulation controls, providing:

* Open hardware designs
* Open firmware
* Manufacturing files
* Assembly documentation
* Community-driven improvements

Contributions, feedback, testing results, and suggestions are always welcome.
 firmware, and mechanical integration are evolving, and improvements are being made continuously.

Expect changes in:

PCB revisions
Firmware architecture
Mechanical design and mounting
Feature expansion (throttle, buttons, auxiliary controls, twist/rudder axis)
