//! EAPOL-Key: the four frames of the WPA2 handshake, parsed and built.
//!
//! This is the piece that makes a radio a network, and it is where the arithmetic has to be exact. The
//! frame is a flat byte layout (802.1X carries EAPOL, EAPOL-Key carries the RSN descriptor), and the two
//! things a wrong implementation gets wrong are worth naming up front:
//!
//!   * The MIC covers the whole EAPOL frame **with the MIC field itself zeroed**. Computing it over the
//!     frame as received, with whatever was in that field, produces a frame the AP discards -- and a
//!     discarded frame is silence, not an error.
//!   * Message 3's key data is encrypted with the KEK when the encrypted-key-data bit is set, and the
//!     integrity word inside AES key-wrap is what catches a PTK derived from the wrong nonces. So the
//!     unwrap must REFUSE rather than return a plausible-looking GTK.
//!
//! Frame layout (802.1X-2010 + 802.11i), all multi-byte fields big-endian:
//!
//!   EAPOL:  version(1)=2  type(1)=3  length(2) = 95 + key_data_len
//!   Key  :  descriptor_type(1)=2  key_info(2)  key_length(2)  replay(8)  nonce(32)  iv(16)
//!           rsc(8)  key_id(8)  mic(16)  key_data_len(2)  key_data(n)

use crate::crypto::*;

pub const EAPOL_VERSION: u8 = 2;
pub const EAPOL_TYPE_KEY: u8 = 3;
pub const DESCRIPTOR_RSN: u8 = 2;
/// 1 (HMAC-SHA1 + AES key wrap), the version WPA2-PSK uses with CCMP.
pub const KEY_DESC_VERSION: u16 = 2;
pub const MIC_LEN: usize = 16;
/// The body from descriptor_type up to and including key_data_len: 95 bytes, with no key data.
pub const KEY_BODY_FIXED: usize = 95;
pub const KDE_TYPE: u8 = 0xdd;
pub const KDE_OUI: [u8; 3] = [0x00, 0x0f, 0xac];
pub const KDE_GTK: u8 = 4;
pub const ETHERTYPE_EAPOL: u16 = 0x888e;

// key_info bits
pub const KI_KEY_TYPE_PAIRWISE: u16 = 0x0008;
pub const KI_INSTALL: u16 = 0x0040;
pub const KI_ACK: u16 = 0x0080;
pub const KI_MIC: u16 = 0x0100;
pub const KI_SECURE: u16 = 0x0200;
pub const KI_ENCRYPTED: u16 = 0x1000;
pub const KI_REQUEST: u16 = 0x0800;

#[derive(Debug, Clone)]
pub struct EapolKey {
    pub key_info: u16,
    pub key_length: u16,
    pub replay: u64,
    pub nonce: [u8; 32],
    pub mic: [u8; MIC_LEN],
    pub key_data: Vec<u8>,
    /// The whole frame, so the MIC can be computed over it with the MIC field zeroed.
    pub raw: Vec<u8>,
}

impl EapolKey {
    pub fn is_pairwise(&self) -> bool {
        self.key_info & KI_KEY_TYPE_PAIRWISE != 0
    }

    pub fn has_mic(&self) -> bool {
        self.key_info & KI_MIC != 0
    }

    pub fn is_encrypted_key_data(&self) -> bool {
        self.key_info & KI_ENCRYPTED != 0
    }

    pub fn version(&self) -> u16 {
        self.key_info & 0x0007
    }

    /// Message number in the handshake, from the bits the standard defines. 1 and 3 come from the AP,
    /// 2 and 4 from here, and telling them apart is the whole of the state machine's input.
    pub fn handshake_message(&self) -> u8 {
        match (
            self.key_info & KI_ACK != 0,
            self.key_info & KI_MIC != 0,
            self.key_info & KI_SECURE != 0,
            self.key_info & KI_INSTALL != 0,
        ) {
            (true, false, _, _) => 1,
            (false, true, false, _) => 2,
            (true, true, true, _) => 3,
            (false, true, true, _) => 4,
            _ => 0,
        }
    }

    /// The MIC over this frame as it must be computed: the MIC field zeroed, everything else as it
    /// stands. This is the single most common way a handshake silently fails.
    pub fn mic_input(&self) -> Vec<u8> {
        let mut f = self.raw.clone();
        let mic_at = 4 + 1 + 2 + 2 + 8 + 32 + 16 + 8 + 8;
        if f.len() >= mic_at + MIC_LEN {
            for b in &mut f[mic_at..mic_at + MIC_LEN] {
                *b = 0;
            }
        }
        f
    }
}

/// Parse an EAPOL-Key frame. Anything that is not one is not an error: this program listens on a packet
/// socket and will see other traffic, so a non-EAPOL frame is simply not this.
pub fn parse(frame: &[u8]) -> Option<EapolKey> {
    // With a data link header in front, EAPOL starts after 14 bytes; the caller decides which it has.
    for start in [0usize, 14] {
        if frame.len() < start + 4 {
            continue;
        }
        let b = &frame[start..];
        if b[0] != EAPOL_VERSION || b[1] != EAPOL_TYPE_KEY {
            continue;
        }
        let declared = u16::from_be_bytes([b[2], b[3]]) as usize;
        if b.len() < 4 + KEY_BODY_FIXED || declared < KEY_BODY_FIXED {
            continue;
        }
        let body = &b[4..];
        if body[0] != DESCRIPTOR_RSN {
            continue;
        }
        let key_info = u16::from_be_bytes([body[1], body[2]]);
        let key_length = u16::from_be_bytes([body[3], body[4]]);
        let replay = u64::from_be_bytes(body[5..13].try_into().ok()?);
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&body[13..45]);
        let mut mic = [0u8; MIC_LEN];
        mic.copy_from_slice(&body[77..93]);
        let kd_len = u16::from_be_bytes([body[93], body[94]]) as usize;
        let kd = if body.len() >= KEY_BODY_FIXED + kd_len {
            body[KEY_BODY_FIXED..KEY_BODY_FIXED + kd_len].to_vec()
        } else {
            Vec::new()
        };
        // `raw` is the EAPOL frame itself (no data link header), which is what the MIC covers. Its length
        // is taken before the move below, because a moved value cannot be measured afterwards.
        let raw_len = 4 + KEY_BODY_FIXED + kd.len();
        return Some(EapolKey {
            key_info,
            key_length,
            replay,
            nonce,
            mic,
            key_data: kd,
            raw: b[..raw_len].to_vec(),
        });
    }
    None
}

/// Build message 2 or 4 from here. `mic_ptk` is the PTK's first 16 bytes; the MIC field is written as
/// zeros, the digest is taken, and the result is placed in the frame.
pub fn build(
    replay: u64,
    nonce: &[u8; 32],
    key_info: u16,
    key_data: &[u8],
    kck: &[u8],
) -> Vec<u8> {
    let mut body = Vec::with_capacity(KEY_BODY_FIXED + key_data.len());
    body.push(DESCRIPTOR_RSN);
    body.extend_from_slice(&key_info.to_be_bytes());
    body.extend_from_slice(&16u16.to_be_bytes()); // key length: 16 for CCMP
    body.extend_from_slice(&replay.to_be_bytes());
    body.extend_from_slice(nonce);
    body.extend_from_slice(&[0u8; 16]); // iv
    body.extend_from_slice(&[0u8; 8]); // rsc
    body.extend_from_slice(&[0u8; 8]); // key id
    body.extend_from_slice(&[0u8; MIC_LEN]); // mic, zeroed for the digest
    body.extend_from_slice(&(key_data.len() as u16).to_be_bytes());
    body.extend_from_slice(key_data);

    let mut frame = vec![EAPOL_VERSION, EAPOL_TYPE_KEY];
    frame.extend_from_slice(&(body.len() as u16).to_be_bytes());
    frame.extend_from_slice(&body);

    // The digest covers the frame with the MIC field zeroed -- which it currently is.
    let mic = eapol_mic(kck, &frame);
    frame[4 + 77..4 + 77 + MIC_LEN].copy_from_slice(&mic);
    frame
}

/// The key data of message 3 as a plaintext list of key data encapsulations. When the frame set the
/// encrypted bit the key data arrives AES-key-wrapped, so this must unwrap first -- and a KEK from the
/// wrong PTK fails the wrap's integrity check rather than returning a wrong GTK.
pub fn gtk_from_key_data(key_data: &[u8], kek: &[u8], encrypted: bool) -> Option<Vec<u8>> {
    let plain = if encrypted {
        aes_kw_unwrap(kek, key_data)?
    } else {
        key_data.to_vec()
    };
    let mut i = 0usize;
    while i + 2 <= plain.len() {
        let ty = plain[i];
        let len = plain[i + 1] as usize;
        if len < 2 || i + 2 + len > plain.len() {
            return None;
        }
        let data = &plain[i + 2..i + 2 + len];
        if ty == KDE_TYPE && data.len() >= 6 && data[..3] == KDE_OUI {
            if data[3] == KDE_GTK {
                // key id(1) reserved(1) then the key itself
                return Some(data[6..].to_vec());
            }
        }
        i += 2 + len;
    }
    None
}

/// The RSN information element a station echoes in message 2. Copied from the AP's own beacon or from
/// message 1 of the handshake -- never invented, because the AP compares it.
pub fn rsn_ie_from(key_data: &[u8]) -> Vec<u8> {
    let mut i = 0usize;
    while i + 2 <= key_data.len() {
        let ty = key_data[i];
        let len = key_data[i + 1] as usize;
        if i + 2 + len > key_data.len() {
            break;
        }
        if ty == 0x30 {
            return key_data[i + 2..i + 2 + len].to_vec();
        }
        i += 2 + len;
    }
    Vec::new()
}
