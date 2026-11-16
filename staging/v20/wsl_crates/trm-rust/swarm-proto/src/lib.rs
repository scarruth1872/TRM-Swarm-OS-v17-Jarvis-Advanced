//! swarm-proto — the trm:// protocol: K3 ternary binary framing.
//!
//! Replaces HTTP/JSON/WebSocket on the wire with a 4-byte header +
//! ternary payload. Every message is a K3 decision vector:
//!
//!   frame := magic u8 (0x4B) | kind u8 | len u16 LE | payload (2 bits/trit)
//!
//! Kinds: 0x01=GREETING 0x02=QUERY 0x03=RESPONSE 0x04=VERDICT
//!        0x05=PING 0x06=PONG 0x07=K3_VERDICT (ternary result)
//!
//! Payload trits use the canonical K3 codec (Phase 2 unification):
//!   FALSE(-1)→0b00, UNKNOWN(0)→0b01, TRUE(+1)→0b11.

use byteorder::{ByteOrder, LittleEndian};

pub const MAGIC: u8 = 0x4B; // 'K'
pub const HEADER_LEN: usize = 4;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Greeting = 0x01,
    Query = 0x02,
    Response = 0x03,
    Verdict = 0x04,
    Ping = 0x05,
    Pong = 0x06,
    K3Verdict = 0x07,
    KvGet = 0x08,
    KvSet = 0x09,
    KvDel = 0x0A,
    KvResult = 0x0B,
    Fit = 0x0C, // Inspired Fitness business records over trm:// (no HTTP)
}

impl Kind {
    pub fn from_u8(v: u8) -> Option<Kind> {
        match v {
            0x01 => Some(Kind::Greeting),
            0x02 => Some(Kind::Query),
            0x03 => Some(Kind::Response),
            0x04 => Some(Kind::Verdict),
            0x05 => Some(Kind::Ping),
            0x06 => Some(Kind::Pong),
            0x07 => Some(Kind::K3Verdict),
            0x08 => Some(Kind::KvGet),
            0x09 => Some(Kind::KvSet),
            0x0A => Some(Kind::KvDel),
            0x0B => Some(Kind::KvResult),
            0x0C => Some(Kind::Fit),
            _ => None,
        }
    }
}

/// Encode a frame: header + packed ternary payload.
pub fn encode(kind: Kind, trits: &[i8]) -> Vec<u8> {
    let n = trits.len();
    let payload_len = (n + 3) / 4;
    let mut out = Vec::with_capacity(HEADER_LEN + payload_len);
    out.push(MAGIC);
    out.push(kind as u8);
    let mut lenbuf = [0u8; 2];
    LittleEndian::write_u16(&mut lenbuf, n as u16);
    out.extend_from_slice(&lenbuf);
    out.extend_from_slice(&pack_trits(trits));
    out
}

/// Pack trits to canonical K3 2-bit codes (matches swarm-turbovec).
pub fn pack_trits(trits: &[i8]) -> Vec<u8> {
    let nbytes = (trits.len() + 3) / 4;
    let mut out = vec![0u8; nbytes];
    for (i, &t) in trits.iter().enumerate() {
        let code: u8 = match t {
            1 => 3,
            -1 => 0,
            _ => 1,
        };
        out[i / 4] |= code << ((i % 4) * 2);
    }
    out
}

/// Unpack canonical K3 2-bit codes back to trits.
pub fn unpack_trits(bytes: &[u8], n_trits: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(n_trits);
    for (i, &b) in bytes.iter().enumerate() {
        for k in 0..4 {
            if i * 4 + k >= n_trits {
                break;
            }
            out.push(match (b >> (k * 2)) & 0b11 {
                0 => -1,
                3 => 1,
                _ => 0,
            });
        }
    }
    out
}

/// Encode UTF-8 text as trits: bit 1 -> +1, bit 0 -> -1 (MSB first).
/// Carries trm:// text commands (e.g. fit:booking:...) over the K3 frame.
pub fn text_to_trits(s: &str) -> Vec<i8> {
    let mut out = Vec::with_capacity(s.len() * 8);
    for &b in s.as_bytes() {
        for k in (0..8).rev() {
            out.push(if (b >> k) & 1 == 1 { 1 } else { -1 });
        }
    }
    out
}

/// Decode trits back to UTF-8 text: +1 -> bit 1, anything else -> bit 0.
pub fn trits_to_text(trits: &[i8]) -> String {
    let mut bytes = Vec::with_capacity(trits.len() / 8 + 1);
    for chunk in trits.chunks(8) {
        let mut b: u8 = 0;
        for (k, &t) in chunk.iter().enumerate() {
            if t == 1 {
                b |= 1 << (7 - k);
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Parse a frame from a byte buffer. Returns (kind, trits, consumed).
pub fn decode(buf: &[u8]) -> Option<(Kind, Vec<i8>, usize)> {
    if buf.len() < HEADER_LEN {
        return None;
    }
    if buf[0] != MAGIC {
        return None;
    }
    let kind = Kind::from_u8(buf[1])?;
    let n = LittleEndian::read_u16(&buf[2..4]) as usize;
    let payload_len = (n + 3) / 4;
    let total = HEADER_LEN + payload_len;
    if buf.len() < total {
        return None;
    }
    let trits = unpack_trits(&buf[HEADER_LEN..total], n);
    Some((kind, trits, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_k3_verdict() {
        let trits = [1, 0, -1, 1, 1, -1, 0, 1, -1, 0, 1, 1];
        let frame = encode(Kind::K3Verdict, &trits);
        assert_eq!(frame[0], MAGIC);
        let (kind, back, consumed) = decode(&frame).unwrap();
        assert_eq!(kind, Kind::K3Verdict);
        assert_eq!(back, trits);
        assert_eq!(consumed, frame.len());
    }

    #[test]
    fn greeting_frame_shape() {
        let frame = encode(Kind::Greeting, &[1, 1, 1]);
        assert_eq!(frame.len(), HEADER_LEN + 1);
        assert_eq!(&frame[..2], &[MAGIC, 0x01]);
    }

    #[test]
    fn canonical_codes_match_turbovec() {
        // same byte pattern as swarm_turbovec::pack([-1,0,1,-1,0,1,-1,1])
        let p = pack_trits(&[-1, 0, 1, -1, 0, 1, -1, 1]);
        assert_eq!(p[0], 0b00 | (0b01 << 2) | (0b11 << 4) | (0b00 << 6));
        assert_eq!(p[1], 0b01 | (0b11 << 2) | (0b00 << 4) | (0b11 << 6));
    }
}
