//! SHA-256 and HMAC-SHA256, dependency-free, for the no_std kernel.
//!
//! Why hand-rolled: the kernel has zero dependencies by doctrine, so `sha2`/`hmac`
//! cannot be used. This file implements FIPS 180-4 SHA-256 and RFC 2104 HMAC from
//! the specification. Correctness is pinned by test vectors (FIPS 180-4 examples
//! and RFC 4231 HMAC cases) - see hmac_sha256_vectors.md.
//!
//! Purpose: authenticate TRM_MESH beacons. `mesh_discovery.rs` currently accepts a
//! peer when `magic` and `truth_anchor` match, and BOTH are public constants, so any
//! LAN host can forge enrollment. The signed form appends HMAC-SHA256(key, body).

/// FIPS 180-4 round constants: fractional parts of the cube roots of the first 64 primes.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 initial hash value: fractional parts of the square roots of the first 8 primes.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Streaming SHA-256 state.
pub struct Sha256 {
    h: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256 {
    pub const fn new() -> Self {
        Sha256 { h: H0, buf: [0u8; 64], buf_len: 0, total_len: 0 }
    }

    /// Process one 64-byte block.
    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d) = (self.h[0], self.h[1], self.h[2], self.h[3]);
        let (mut e, mut f, mut g, mut hh) = (self.h[4], self.h[5], self.h[6], self.h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        self.h[0] = self.h[0].wrapping_add(a); self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c); self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e); self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g); self.h[7] = self.h[7].wrapping_add(hh);
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        if self.buf_len > 0 {
            let need = 64 - self.buf_len;
            let take = if data.len() < need { data.len() } else { need };
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let blk = self.buf;
                self.compress(&blk);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let (blk, rest) = data.split_at(64);
            self.compress(blk);
            data = rest;
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    /// Finalise and return the 32-byte digest. Consumes the state.
    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len.wrapping_mul(8);
        self.update(&[0x80]);
        while self.buf_len != 56 {
            self.update(&[0x00]);
        }
        // bit_len was captured before padding, which is what FIPS 180-4 requires:
        // the length field counts the ORIGINAL message bits, not the padding.
        let len_bytes = bit_len.to_be_bytes();
        self.buf[56..64].copy_from_slice(&len_bytes);
        let b = self.buf;
        self.compress(&b);
        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i*4..i*4+4].copy_from_slice(&self.h[i].to_be_bytes());
        }
        out
    }
}

/// One-shot SHA-256.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut s = Sha256::new();
    s.update(data);
    s.finalize()
}

/// RFC 2104 HMAC-SHA256.
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(msg);
    let ih = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&ih);
    outer.finalize()
}

/// Constant-time compare, so tag verification does not leak position of mismatch.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff = 0u8;
    for i in 0..a.len() { diff |= a[i] ^ b[i]; }
    diff == 0
}

// ── Signed TRM_MESH beacon (shared format with the host side) ──────────────
//
//   body = 36 bytes, byte-identical to the existing mesh_discovery.rs layout
//   tag  = HMAC-SHA256(key, body), 32 bytes
//   wire = body || tag  (68 bytes)
//
// The body is unchanged so an existing receiver still parses it; the tag is what
// makes forgery detectable. Acceptance = tag verifies AND magic AND anchor match.

pub const MESH_MAGIC: u64 = 0x5452_4D5F_4D45_5348; // "TRM_MESH"
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
pub const BEACON_BODY: usize = 36;
pub const BEACON_TAG: usize = 32;
pub const BEACON_SIGNED: usize = BEACON_BODY + BEACON_TAG;

/// Serialise the 36-byte body exactly as mesh_discovery.rs does.
pub fn build_beacon_body(node_id: u64, agent_count: u16, trit_count: u32, ts_ms: u64) -> [u8; BEACON_BODY] {
    let mut b = [0u8; BEACON_BODY];
    b[0..8].copy_from_slice(&MESH_MAGIC.to_le_bytes());
    b[8..16].copy_from_slice(&node_id.to_le_bytes());
    b[16..24].copy_from_slice(&MONAD_TRUTH_ANCHOR.to_le_bytes());
    b[24..26].copy_from_slice(&agent_count.to_le_bytes());
    b[26..30].copy_from_slice(&trit_count.to_le_bytes());
    // NOTE: mesh_discovery.rs writes only 6 of 8 timestamp bytes here.
    b[30..36].copy_from_slice(&ts_ms.to_le_bytes()[..6]);
    b
}

/// Sign a body, returning the 68-byte wire form.
pub fn sign_beacon(body: &[u8; BEACON_BODY], key: &[u8]) -> [u8; BEACON_SIGNED] {
    let mut out = [0u8; BEACON_SIGNED];
    out[..BEACON_BODY].copy_from_slice(body);
    let tag = hmac_sha256(key, body);
    out[BEACON_BODY..].copy_from_slice(&tag);
    out
}

/// Verify a signed beacon. Returns Ok((node_id, agent_count, trit_count)) or Err(reason).
pub fn verify_beacon(blob: &[u8], key: &[u8]) -> Result<(u64, u16, u32), &'static str> {
    if blob.len() < BEACON_SIGNED {
        return Err("beacon too short");
    }
    let body = &blob[..BEACON_BODY];
    let tag = &blob[BEACON_BODY..BEACON_SIGNED];
    let expect = hmac_sha256(key, body);
    if !ct_eq(tag, &expect) {
        return Err("HMAC mismatch - forged or wrong key");
    }
    let magic = u64::from_le_bytes([body[0],body[1],body[2],body[3],body[4],body[5],body[6],body[7]]);
    if magic != MESH_MAGIC { return Err("bad magic"); }
    let anchor = u64::from_le_bytes([body[16],body[17],body[18],body[19],body[20],body[21],body[22],body[23]]);
    if anchor != MONAD_TRUTH_ANCHOR { return Err("bad truth anchor"); }
    let node_id = u64::from_le_bytes([body[8],body[9],body[10],body[11],body[12],body[13],body[14],body[15]]);
    let agents = u16::from_le_bytes([body[24], body[25]]);
    let trits = u32::from_le_bytes([body[26], body[27], body[28], body[29]]);
    Ok((node_id, agents, trits))
}
