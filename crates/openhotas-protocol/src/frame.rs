//! CDC binary frame format.
//!
//! ## Frame structure (D-02 — request/response, no spontaneous telemetry)
//!
//! ```text
//! Offset  Size  Field
//! ─────────────────────────────────────
//! 0       2     SOF = 0xAA 0x55
//! 2       2     LEN u16 big-endian
//! 4       LEN   PAYLOAD (postcard-serialized Request or Response)
//! 4+LEN   2     CRC16-CCITT big-endian (covers LEN + PAYLOAD)
//! ```
//!
//! ## CRC coverage
//! CRC16 covers `LEN + PAYLOAD`, not just PAYLOAD.
//! Reason (D-02): protects the length field. If LEN is corrupted,
//! the parser has a better chance of rejecting the frame before
//! deserializing the wrong number of bytes.
//!
//! ## Realignment
//! Any error resets to scanning for SOF (AA55). The parser never panics.

pub const SOF_A: u8 = 0xAA;
pub const SOF_B: u8 = 0x55;
pub const MAX_PAYLOAD_SIZE: usize = 256;

// CRC16-CCITT parameters
const CRC_POLY: u16 = 0x1021;
const CRC_INIT: u16 = 0xFFFF;

/// Computes CRC16-CCITT over `data`.
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = CRC_INIT;
    for &byte in data {
        crc = crc16_update(crc, byte);
    }
    crc
}

/// Feed one byte into a running CRC16 calculation.
fn crc16_update(crc: u16, byte: u8) -> u16 {
    let mut crc = crc ^ ((byte as u16) << 8);
    for _ in 0..8 {
        if crc & 0x8000 != 0 {
            crc = (crc << 1) ^ CRC_POLY;
        } else {
            crc <<= 1;
        }
    }
    crc
}

/// CDC frame parser with internal buffer and CRC accumulator.
///
/// Feed one byte at a time via `feed()`. Returns `Some(frame)` when
/// a complete valid frame is received. On any error (CRC mismatch,
/// invalid length), discards the partial frame and resets to scanning
/// for SOF.
pub struct FrameParser {
    state: State,
    /// Accumulated payload bytes (up to MAX_PAYLOAD_SIZE).
    buf: [u8; MAX_PAYLOAD_SIZE],
    /// Expected payload length (set during ReadLenLo).
    payload_len: usize,
    /// Running CRC. Covers LEN (2 bytes) + payload bytes as they arrive.
    crc: u16,
    /// CRC high byte received, waiting for low byte.
    crc_hi: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    WaitSofA,
    WaitSofB,
    ReadLenHi,
    ReadLenLo { hi: u8 },
    ReadPayload { len: usize, pos: usize },
    ReadCrcHi,
    ReadCrcLo { hi: u8 },
}

/// Frame-level parse error.
///
/// V1.23: replacing `Infallible` so the firmware can count protocol CRC errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// CRC16-CCITT mismatch on LEN + PAYLOAD.
    CrcMismatch,
    /// LEN field exceeds MAX_PAYLOAD_SIZE.
    InvalidLength,
}

/// Successful parse result: the raw payload bytes.
#[derive(Debug, PartialEq)]
pub struct ParsedFrame {
    pub payload: heapless::Vec<u8, MAX_PAYLOAD_SIZE>,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            state: State::WaitSofA,
            buf: [0u8; MAX_PAYLOAD_SIZE],
            payload_len: 0,
            crc: CRC_INIT,
            crc_hi: 0,
        }
    }

    /// Feed one byte. Returns `Some(frame)` on complete valid frame,
    /// `None` if more bytes are needed. Never panics.
    pub fn feed(&mut self, byte: u8) -> Result<Option<ParsedFrame>, FrameError> {
        match self.state {
            State::WaitSofA => {
                if byte == SOF_A {
                    self.state = State::WaitSofB;
                }
                Ok(None)
            }
            State::WaitSofB => {
                if byte == SOF_B {
                    self.state = State::ReadLenHi;
                } else {
                    // Could be another SOF_A starting a new frame
                    self.state = if byte == SOF_A {
                        State::WaitSofB
                    } else {
                        State::WaitSofA
                    };
                }
                Ok(None)
            }
            State::ReadLenHi => {
                self.state = State::ReadLenLo { hi: byte };
                Ok(None)
            }
            State::ReadLenLo { hi } => {
                let len = u16::from_be_bytes([hi, byte]) as usize;
                if len > MAX_PAYLOAD_SIZE {
                    self.reset();
                    return Err(FrameError::InvalidLength);
                }
                // Start CRC with LEN bytes
                self.payload_len = len;
                self.crc = CRC_INIT;
                self.crc = crc16_update(self.crc, hi);
                self.crc = crc16_update(self.crc, byte);
                if len == 0 {
                    self.state = State::ReadCrcHi;
                } else {
                    self.state = State::ReadPayload { len, pos: 0 };
                }
                Ok(None)
            }
            State::ReadPayload { len, pos } => {
                self.buf[pos] = byte;
                self.crc = crc16_update(self.crc, byte);
                let new_pos = pos + 1;
                if new_pos >= len {
                    self.state = State::ReadCrcHi;
                } else {
                    self.state = State::ReadPayload { len, pos: new_pos };
                }
                Ok(None)
            }
            State::ReadCrcHi => {
                self.crc_hi = byte;
                self.state = State::ReadCrcLo { hi: byte };
                Ok(None)
            }
            State::ReadCrcLo { .. } => {
                let expected_crc = u16::from_be_bytes([self.crc_hi, byte]);
                if self.crc == expected_crc {
                    let mut payload = heapless::Vec::new();
                    // Copy payload bytes from buf
                    for &b in &self.buf[..self.payload_len] {
                        if payload.push(b).is_err() {
                            self.reset();
                            return Ok(None);
                        }
                    }
                    let frame = ParsedFrame { payload };
                    self.reset();
                    Ok(Some(frame))
                } else {
                    self.reset();
                    Err(FrameError::CrcMismatch)
                }
            }
        }
    }

    fn reset(&mut self) {
        self.state = State::WaitSofA;
        self.crc = CRC_INIT;
        self.crc_hi = 0;
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    /// Helper: build a raw frame byte slice for testing.
    fn build_frame_bytes(payload: &[u8]) -> Vec<u8, 300> {
        let mut frame = Vec::new();
        frame.push(SOF_A).ok();
        frame.push(SOF_B).ok();
        let len = (payload.len() as u16).to_be_bytes();
        frame.extend_from_slice(&len).ok();
        frame.extend_from_slice(payload).ok();
        // CRC covers LEN + PAYLOAD
        let mut crc_input = Vec::<u8, { 2 + MAX_PAYLOAD_SIZE }>::new();
        crc_input.extend_from_slice(&len).ok();
        crc_input.extend_from_slice(payload).ok();
        let crc = crc16_ccitt(&crc_input);
        frame.extend_from_slice(&crc.to_be_bytes()).ok();
        frame
    }

    #[test]
    fn valid_frame_roundtrip() {
        let payload = [1u8, 2, 3, 4];
        let frame = build_frame_bytes(&payload);
        let mut parser = FrameParser::new();

        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(&parsed.payload[..], &payload[..]);
    }

    #[test]
    fn invalid_sof_rejected() {
        let mut parser = FrameParser::new();
        parser.feed(0x00).unwrap();
        parser.feed(SOF_A).unwrap();
        parser.feed(0x00).unwrap(); // Not SOF_B — should reset
                                    // Feed valid frame to prove parser recovered
        let payload = [42u8];
        let frame = build_frame_bytes(&payload);
        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }
        assert!(result.is_some());
    }

    #[test]
    fn crc_mismatch_rejected() {
        let payload = [1u8, 2, 3];
        let mut frame = build_frame_bytes(&payload);
        // Corrupt the last byte (CRC)
        let last = frame.len() - 1;
        frame[last] ^= 0xFF;

        let mut parser = FrameParser::new();
        let mut final_result = Ok(None);
        for &byte in &frame {
            final_result = parser.feed(byte);
        }
        assert_eq!(final_result, Err(FrameError::CrcMismatch));
    }

    #[test]
    fn len_exceeds_max_rejected() {
        let mut parser = FrameParser::new();
        parser.feed(SOF_A).unwrap();
        parser.feed(SOF_B).unwrap();
        // LEN = 257 (> MAX_PAYLOAD_SIZE = 256)
        parser.feed(0x01).unwrap(); // hi byte of 257
        let result = parser.feed(0x01); // lo byte — should return InvalidLength
        assert_eq!(result, Err(FrameError::InvalidLength));
        // Parser should reset — prove by feeding a valid frame
        let payload = [55u8];
        let frame = build_frame_bytes(&payload);
        let mut final_result = Ok(None);
        for &byte in &frame {
            final_result = parser.feed(byte);
        }
        assert!(final_result.is_ok_and(|r| r.is_some()));
    }

    #[test]
    fn realignment_after_garbage() {
        let mut parser = FrameParser::new();
        for b in [0xFF, 0x00, 0xAB, 0xCD] {
            parser.feed(b).unwrap();
        }
        let payload = [88u8, 99];
        let frame = build_frame_bytes(&payload);
        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }
        assert!(result.is_some());
    }

    #[test]
    fn sof_a_in_0xaa_payload_not_misinterpreted() {
        // 0xAA in payload should not trigger SOF detection mid-parse
        let payload = [SOF_A, 0xBB, 0xCC];
        let frame = build_frame_bytes(&payload);
        let mut parser = FrameParser::new();
        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(&parsed.payload[..], &payload[..]);
    }

    #[test]
    fn zero_length_payload() {
        let payload: [u8; 0] = [];
        let frame = build_frame_bytes(&payload);
        let mut parser = FrameParser::new();
        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(parsed.payload.is_empty());
    }

    #[test]
    fn max_length_payload() {
        let payload = [0xABu8; MAX_PAYLOAD_SIZE];
        let frame = build_frame_bytes(&payload);
        let mut parser = FrameParser::new();
        let mut result = None;
        for &byte in &frame {
            result = parser.feed(byte).unwrap();
        }
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.payload.len(), MAX_PAYLOAD_SIZE);
    }
}
