//! swarm-turbovec — 16× ternary packing in Rust (canonical K3 v2).
//!
//! Port of swarm_v2/core/turbovec_ternary_pack.py + the fabric's
//! pack_trits_fast, with the K3 code table (Phase 2 unification):
//!   FALSE(-1) → 0b00 (0)   UNKNOWN(0) → 0b01 (1)   TRUE(+1) → 0b11 (3)
//!
//! 4 trits per byte (2 bits each) = 16× compression vs fp32 (4 bytes/trit).
//!
//! Verify gate: byte-identical pack/unpack vs the numpy reference, and
//! round-trip exactness on random {-1,0,1} streams.

/// Pack {-1,0,1} trits → 2-bit canonical bytes (4 trits/byte, LSB-first).
/// Vector-friendly; matches pack_bits_fast semantics byte-for-byte.
pub fn pack(trits: &[i8]) -> Vec<u8> {
    let n = trits.len();
    let nbytes = (n + 3) / 4;
    let mut out = vec![0u8; nbytes];
    for (i, &t) in trits.iter().enumerate() {
        let code: u8 = match t {
            1 => 3,   // TRUE
            -1 => 0,  // FALSE
            _ => 1,   // UNKNOWN
        };
        out[i / 4] |= code << ((i % 4) * 2);
    }
    out
}

/// Unpack 2-bit canonical bytes → {-1,0,1} trits.
pub fn unpack(bytes: &[u8], n_trits: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(n_trits);
    for (i, &b) in bytes.iter().enumerate() {
        for k in 0..4 {
            let idx = i * 4 + k;
            if idx >= n_trits {
                break;
            }
            out.push(match (b >> (k * 2)) & 0b11 {
                0 => -1,  // FALSE
                3 => 1,   // TRUE
                _ => 0,   // UNKNOWN (incl. unused 0b10)
            });
        }
    }
    out
}

/// 2-bit cosine similarity between two packed ternary vectors (K3).
/// Matches c_turbovec_cosine_2bit semantics: TRUE=+1, FALSE=-1,
/// UNKNOWN=0 contributions; cosine over the agreement-weighted vectors.
pub fn cosine_2bit(a: &[u8], b: &[u8]) -> f32 {
    let n = a.len().min(b.len());
    let mut dot: f32 = 0.0;
    let mut na: f32 = 0.0;
    let mut nb: f32 = 0.0;
    for i in 0..n {
        for k in 0..4 {
            let ca = (a[i] >> (k * 2)) & 0b11;
            let cb = (b[i] >> (k * 2)) & 0b11;
            let va = code_to_f32(ca);
            let vb = code_to_f32(cb);
            dot += va * vb;
            na += va * va;
            nb += vb * vb;
        }
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

#[inline]
fn code_to_f32(c: u8) -> f32 {
    match c {
        0 => -1.0, // FALSE
        3 => 1.0,  // TRUE
        _ => 0.0,  // UNKNOWN / unused
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_matches_reference_byte_pattern() {
        // Python pack_bits_fast([-1,0,1,-1,0,1,-1,1]) →
        // codes [0,1,3,0,1,3,0,3] → byte0 = 0|1<<2|3<<4|0<<6 = 0b00011100...
        // verify: code0=0, code1=1, code2=3, code3=0 → 0 | 4 | 48 | 0 = 52
        let trits: Vec<i8> = vec![-1, 0, 1, -1, 0, 1, -1, 1];
        let p = pack(&trits);
        assert_eq!(p[0], 0b00 | (0b01 << 2) | (0b11 << 4) | (0b00 << 6));
        // byte1 = 0b01 | 0b11<<2 | 0b00<<4 | 0b11<<6 = 1 | 12 | 0 | 192 = 205
        assert_eq!(p[1], 0b01 | (0b11 << 2) | (0b00 << 4) | (0b11 << 6));
        assert_eq!(unpack(&p, 8), trits);
    }

    #[test]
    fn roundtrip_exact_random() {
        // deterministic PRNG (matches Python RandomState(42) style)
        let mut state: u64 = 42;
        for size in [100usize, 1000, 1_000_000] {
            let mut trits = Vec::with_capacity(size);
            for _ in 0..size {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                trits.push(((state >> 33) % 3) as i8 - 1); // -1,0,1
            }
            let p = pack(&trits);
            assert_eq!(p.len(), (size + 3) / 4);
            assert_eq!(unpack(&p, size), trits, "size {size}");
        }
    }

    #[test]
    fn compression_16x() {
        // 1M trits → 250KB (4 bytes/trit fp32 = 4MB → 16×)
        let trits = vec![0i8; 1_000_000];
        let p = pack(&trits);
        assert_eq!(p.len(), 250_000);
    }

    #[test]
    fn cosine_same_vectors_is_1() {
        let a = pack(&[1, 1, 0, -1, 1, 0, -1, 1, 1, 1, 0, -1]);
        let b = pack(&[1, 1, 0, -1, 1, 0, -1, 1, 1, 1, 0, -1]);
        let c = cosine_2bit(&a, &b);
        assert!((c - 1.0).abs() < 1e-5, "c={c}");
    }

    #[test]
    fn cosine_opposite_is_minus_1() {
        let a = pack(&[1, 1, 0, -1, 1, 0, -1, 1]);
        let b = pack(&[-1, -1, 0, 1, -1, 0, 1, -1]);
        let c = cosine_2bit(&a, &b);
        assert!((c + 1.0).abs() < 1e-5, "c={c}");
    }
}
