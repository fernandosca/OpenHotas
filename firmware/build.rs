use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // ── Injetar git hash ───────────────────────────────────────────────
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
        if let Some(ref_path) = head.strip_prefix("ref: ").map(|s| s.trim()) {
            println!("cargo:rerun-if-changed=.git/{}", ref_path);
        }
    }

    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // ── USB device_release (BCD) derivado do Cargo.toml ─────────────────
    // Formato BCD USB: 0xMMNN = major.minor (1.3.0 → 0x0130).
    // Mantém o descritor USB sincronizado com a versão SemVer do crate,
    // evitando o defasamento manual que deixou device_release preso em 1.23
    // entre V1.25 e V1.3.0. O campo patch é ignorado (USB usa só major.minor).
    let major: u8 = env::var("CARGO_PKG_VERSION_MAJOR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let minor: u8 = env::var("CARGO_PKG_VERSION_MINOR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    // Cada nibble é um dígito decimal; major/minor cabem cada um em 1 byte BCD.
    let bcd = bcd_encode(major) << 8 | bcd_encode(minor);
    println!("cargo:rustc-env=USB_DEVICE_RELEASE_BCD=0x{:04X}", bcd);
}

/// Codifica um valor decimal (0..=99) em BCD (1 nibble alto = dezena, 1 nibble baixo = unidade).
/// Ex.: 30 → 0x30, 3 → 0x03.
fn bcd_encode(v: u8) -> u16 {
    let tens = (v / 10).min(9) as u16;
    let units = (v % 10) as u16;
    (tens << 4) | units
}
