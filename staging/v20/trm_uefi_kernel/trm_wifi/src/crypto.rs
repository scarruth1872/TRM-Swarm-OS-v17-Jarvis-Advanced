//! The crypto a WPA2-PSK supplicant actually needs, and nothing else.
//!
//! No CCMP: the four-way handshake derives keys and the *driver* encrypts data frames, so a supplicant
//! never touches a payload. What it does need is SHA1/HMAC (the PSK derivation, the PRF, the MIC),
//! PBKDF2 (passphrase -> PMK) and AES key-wrap (unwrapping the GTK from message 3). All four are small
//! enough to be written out, which is what keeps this binary dependency-free -- and being written out,
//! all four are checked against oracles rather than trusted (see tests/ and scripts/verify_wifi_crypto.py).

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let tmp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*wi);
            e = d; d = c; c = b.rotate_left(30); b = a; a = tmp;
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b); h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d); h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for i in 0..5 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..20].copy_from_slice(&sha1(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = ipad.to_vec();
    inner.extend_from_slice(data);
    let ih = sha1(&inner);
    let mut outer = opad.to_vec();
    outer.extend_from_slice(&ih);
    sha1(&outer)
}

/// PBKDF2-HMAC-SHA1, one block: the PMK is 32 bytes, so the loop is a single block and `dkLen` is
/// exactly the digest -- anything longer would need the block index in the salt, which WPA2 never asks
/// for. The iteration count is the protocol's 4096.
pub fn pbkdf2_sha1_32(pass: &[u8], salt: &[u8], iters: u32) -> [u8; 32] {
    let mut u = hmac_sha1(pass, &[salt, &1u32.to_be_bytes()].concat());
    let mut t = u;
    for _ in 1..iters {
        u = hmac_sha1(pass, &u);
        for i in 0..20 {
            t[i] ^= u[i];
        }
    }
    let mut out = [0u8; 32];
    out[..20].copy_from_slice(&t);
    // Block 2, for bytes 20..32 -- the only place WPA2's PRF-shaped derivation needs a second HMAC.
    let mut salt2 = salt.to_vec();
    salt2.extend_from_slice(&2u32.to_be_bytes());
    let mut u = hmac_sha1(pass, &salt2);
    let mut t2 = u;
    for _ in 1..iters {
        u = hmac_sha1(pass, &u);
        for i in 0..20 {
            t2[i] ^= u[i];
        }
    }
    out[20..32].copy_from_slice(&t2[..12]);
    out
}

/// The 802.11 PRF-n: HMAC-SHA1(key, label || 0x00 || data || counter), concatenated and cut to n
/// bytes. The counter starts at zero -- an off-by-one here produces a PTK that is wrong in every byte
/// and a MIC that never verifies, which is why it is spelled out.
pub fn prf(key: &[u8], label: &[u8], data: &[u8], n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n);
    let mut counter: u8 = 0;
    while out.len() < n {
        let mut input = Vec::with_capacity(label.len() + 1 + data.len() + 1);
        input.extend_from_slice(label);
        input.push(0);
        input.extend_from_slice(data);
        input.push(counter);
        out.extend_from_slice(&hmac_sha1(key, &input));
        counter = counter.wrapping_add(1);
    }
    out.truncate(n);
    out
}

/// The PTK, in the order the standard fixes: the two MACs sorted, then the two nonces sorted.
pub fn derive_ptk(pmk: &[u8; 32], aa: &[u8; 6], spa: &[u8; 6], anonce: &[u8; 32], snonce: &[u8; 32]) -> [u8; 64] {
    let (mac1, mac2) = if aa <= spa { (aa, spa) } else { (spa, aa) };
    let (n1, n2) = if anonce <= snonce { (anonce, snonce) } else { (snonce, anonce) };
    let mut data = Vec::with_capacity(76);
    data.extend_from_slice(mac1);
    data.extend_from_slice(mac2);
    data.extend_from_slice(n1);
    data.extend_from_slice(n2);
    let ptk = prf(pmk, b"Pairwise key expansion", &data, 64);
    let mut out = [0u8; 64];
    out.copy_from_slice(&ptk);
    out
}

/// The MIC over an EAPOL-Key frame: HMAC-SHA1 of the whole frame with the 16 MIC bytes zeroed, cut to
/// 16 bytes.
pub fn eapol_mic(kck: &[u8], frame: &[u8]) -> [u8; 16] {
    let mic = hmac_sha1(kck, frame);
    let mut out = [0u8; 16];
    out.copy_from_slice(&mic[..16]);
    out
}

// ---- AES-128 ----
//
// The S-box is computed, not pasted: 256 table entries would be the largest literal in this file, and a
// computed one is held to the same oracle as everything else here. Only key-wrap is needed -- the GTK in
// message 3 arrives wrapped in the KEK -- and unwrapping is the *inverse* cipher, so both directions
// exist.

fn gf_mul(a: u8, b: u8) -> u8 {
    let (mut a, mut b, mut p) = (a, b, 0u8);
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    p
}

fn gf_inv(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    let mut r = 1u8;
    for _ in 0..254 {
        r = gf_mul(r, a);
    }
    r
}

fn sbox_table() -> [u8; 256] {
    let mut s = [0u8; 256];
    for i in 0..256usize {
        let inv = gf_inv(i as u8);
        let (mut x, mut y) = (inv, 0u8);
        for _ in 0..5 {
            y ^= x;
            x = x.rotate_left(1);
        }
        s[i] = y ^ 0x63;
    }
    s
}

static SBOX: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();
fn sbox() -> &'static [u8; 256] {
    SBOX.get_or_init(sbox_table)
}

fn inv_sbox() -> [u8; 256] {
    let s = sbox();
    let mut inv = [0u8; 256];
    for i in 0..256 {
        inv[s[i] as usize] = i as u8;
    }
    inv
}

fn expand_key(key: &[u8; 16]) -> [[u8; 16]; 11] {
    const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];
    let s = sbox();
    let mut w = [[0u8; 16]; 11];
    w[0].copy_from_slice(key);
    for round in 1..11 {
        let mut prev = w[round - 1];
        let mut t = [prev[12], prev[13], prev[14], prev[15]];
        t.rotate_left(1);
        for b in t.iter_mut() {
            *b = s[*b as usize];
        }
        t[0] ^= RCON[round - 1];
        for i in 0..4 {
            prev[i] ^= t[i];
        }
        for i in 4..16 {
            prev[i] ^= prev[i - 4];
        }
        w[round] = prev;
    }
    w
}

fn add_round_key(s: &mut [u8; 16], k: &[u8; 16]) {
    for i in 0..16 {
        s[i] ^= k[i];
    }
}

fn shift_rows(s: &mut [u8; 16]) {
    let t = *s;
    for r in 0..4 {
        for c in 0..4 {
            s[r + 4 * c] = t[r + 4 * ((c + r) % 4)];
        }
    }
}

fn inv_shift_rows(s: &mut [u8; 16]) {
    let t = *s;
    for r in 0..4 {
        for c in 0..4 {
            s[r + 4 * c] = t[r + 4 * ((c + 4 - r) % 4)];
        }
    }
}

fn mix_columns(s: &mut [u8; 16]) {
    for c in 0..4 {
        let col = [s[4 * c], s[4 * c + 1], s[4 * c + 2], s[4 * c + 3]];
        s[4 * c] = gf_mul(col[0], 2) ^ gf_mul(col[1], 3) ^ col[2] ^ col[3];
        s[4 * c + 1] = col[0] ^ gf_mul(col[1], 2) ^ gf_mul(col[2], 3) ^ col[3];
        s[4 * c + 2] = col[0] ^ col[1] ^ gf_mul(col[2], 2) ^ gf_mul(col[3], 3);
        s[4 * c + 3] = gf_mul(col[0], 3) ^ col[1] ^ col[2] ^ gf_mul(col[3], 2);
    }
}

fn inv_mix_columns(s: &mut [u8; 16]) {
    for c in 0..4 {
        let col = [s[4 * c], s[4 * c + 1], s[4 * c + 2], s[4 * c + 3]];
        s[4 * c] = gf_mul(col[0], 14) ^ gf_mul(col[1], 11) ^ gf_mul(col[2], 13) ^ gf_mul(col[3], 9);
        s[4 * c + 1] = gf_mul(col[0], 9) ^ gf_mul(col[1], 14) ^ gf_mul(col[2], 11) ^ gf_mul(col[3], 13);
        s[4 * c + 2] = gf_mul(col[0], 13) ^ gf_mul(col[1], 9) ^ gf_mul(col[2], 14) ^ gf_mul(col[3], 11);
        s[4 * c + 3] = gf_mul(col[0], 11) ^ gf_mul(col[1], 13) ^ gf_mul(col[2], 9) ^ gf_mul(col[3], 14);
    }
}

pub fn aes128_encrypt_block(key: &[u8; 16], block: &[u8; 16]) -> [u8; 16] {
    let wk = expand_key(key);
    let s = sbox();
    let mut st = *block;
    add_round_key(&mut st, &wk[0]);
    for round in 1..10 {
        for b in st.iter_mut() {
            *b = s[*b as usize];
        }
        shift_rows(&mut st);
        mix_columns(&mut st);
        add_round_key(&mut st, &wk[round]);
    }
    for b in st.iter_mut() {
        *b = s[*b as usize];
    }
    shift_rows(&mut st);
    add_round_key(&mut st, &wk[10]);
    st
}

pub fn aes128_decrypt_block(key: &[u8; 16], block: &[u8; 16]) -> [u8; 16] {
    let wk = expand_key(key);
    let inv = inv_sbox();
    let mut st = *block;
    add_round_key(&mut st, &wk[10]);
    for round in (1..10).rev() {
        inv_shift_rows(&mut st);
        for b in st.iter_mut() {
            *b = inv[*b as usize];
        }
        add_round_key(&mut st, &wk[round]);
        inv_mix_columns(&mut st);
    }
    inv_shift_rows(&mut st);
    for b in st.iter_mut() {
        *b = inv[*b as usize];
    }
    add_round_key(&mut st, &wk[0]);
    st
}

/// RFC 3394 key unwrap: what unseals the GTK in message 3. The 64-bit integrity check word is not
/// decoration -- a KEK from the wrong PTK produces a wrong plaintext, and this is what refuses it.
/// RFC 3394 key wrap: the inverse of `aes_kw_unwrap`. It exists because a message 3 has to be built by
/// something, and building it with anything other than the algorithm the unwrap checks against would make
/// the harness test a fiction. A length the algorithm does not define -- under 16 bytes, or not a multiple
/// of 8 -- is refused rather than padded.
pub fn aes_kw_wrap(kek: &[u8], plain: &[u8]) -> Option<Vec<u8>> {
    if plain.len() < 16 || plain.len() % 8 != 0 || kek.len() < 16 {
        return None;
    }
    let n = plain.len() / 8;
    let mut kek16 = [0u8; 16];
    kek16.copy_from_slice(&kek[..16]);
    let mut a = [0xa6u8; 8];
    let mut r: Vec<[u8; 8]> = (0..n)
        .map(|i| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&plain[i * 8..i * 8 + 8]);
            b
        })
        .collect();
    for j in 0..6u64 {
        for i in 0..n {
            let mut block = [0u8; 16];
            block[..8].copy_from_slice(&a);
            block[8..].copy_from_slice(&r[i]);
            let enc = aes128_encrypt_block(&kek16, &block);
            let t = (n as u64) * j + (i as u64) + 1;
            for k in 0..8 {
                a[k] = enc[k] ^ t.to_be_bytes()[k];
            }
            r[i].copy_from_slice(&enc[8..]);
        }
    }
    let mut out = Vec::with_capacity(8 + plain.len());
    out.extend_from_slice(&a);
    for b in &r {
        out.extend_from_slice(b);
    }
    Some(out)
}

pub fn aes_kw_unwrap(kek: &[u8], wrapped: &[u8]) -> Option<Vec<u8>> {
    if wrapped.len() < 24 || wrapped.len() % 8 != 0 {
        return None;
    }
    let n = wrapped.len() / 8 - 1;
    let mut a = [0u8; 8];
    a.copy_from_slice(&wrapped[0..8]);
    let mut r: Vec<[u8; 8]> = (0..n)
        .map(|i| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&wrapped[8 * (i + 1)..8 * (i + 2)]);
            b
        })
        .collect();
    let mut key = [0u8; 16];
    key.copy_from_slice(&kek[..16]);
    for j in (0..6).rev() {
        for i in (1..=n).rev() {
            let t = (n * j + i) as u64;
            let mut block = [0u8; 16];
            let av = u64::from_be_bytes(a) ^ t;
            block[..8].copy_from_slice(&av.to_be_bytes());
            block[8..].copy_from_slice(&r[i - 1]);
            let b = aes128_decrypt_block(&key, &block);
            a.copy_from_slice(&b[..8]);
            r[i - 1].copy_from_slice(&b[8..]);
        }
    }
    if a != [0xA6u8; 8] {
        return None;
    }
    let mut out = Vec::with_capacity(n * 8);
    for blk in r {
        out.extend_from_slice(&blk);
    }
    Some(out)
}
