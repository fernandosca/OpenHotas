//! openhotas-protocol — Shared types for firmware ↔ PC communication.
//!
//! ## Design decisions (D-03, D-04, D-05, D-06)
//! - **D-04:** Structs are NEVER transmitted raw. All payloads go through postcard.
//! - **D-05:** Uses `serde` + `postcard` — compact binary, `no_std` compatible.
//! - **D-06:** No `f32` in protocol. All fields are scaled integers (permille, i16, etc.)
//!   to avoid NaN, Infinity, and representation differences across architectures.
//!
//! ## Protocol version
//! This crate defines the current protocol version. The firmware validates
//! `protocol_version_major` before accepting any configuration.

#![no_std]

pub mod config;
pub mod diagnostics;
pub mod error;
pub mod frame;
pub mod request;
pub mod response;
pub mod version;
