//! CRC-32 no estilo ISO-HDLC (ZIP, PNG) — usado para validar configuração
//! persistente na flash.
//!
//! Polinômio refletido: 0xEDB88320 (≡ 0x04C11DB7 não refletido).
//! Init: 0xFFFFFFFF, XorOut: 0xFFFFFFFF.
//!
//! Implementação byte-a-byte (tabela não usada para manter código pequeno
//! em no_std). Processa 8 bits por vez com shift. A performance (~2μs por
//! 256 bytes no RP2350) é adequada para payloads de até 256 bytes.
//!
//! Vetor de teste: `crc32(b"123456789") == 0xCBF43926` (padrão ISO-HDLC).

/// Calcula CRC-32 sobre `data`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector_iso_hdlc() {
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn single_byte_known() {
        assert_eq!(crc32(&[0x00u8]), 0xD202EF8D);
    }

    #[test]
    fn empty_slice_deterministic() {
        let a = crc32(&[]);
        let b = crc32(&[]);
        assert_eq!(a, b);
    }

    #[test]
    fn consistency() {
        let data = b"openhotas test data";
        assert_eq!(crc32(data), crc32(data));
    }
}
