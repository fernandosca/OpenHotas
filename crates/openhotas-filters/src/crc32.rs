/// CRC-32 (ISO-HDLC / ZIP / PNG standard).
/// Polynomial: 0xEDB88320 (reflected), Init: 0xFFFFFFFF, XorOut: 0xFFFFFFFF.
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
