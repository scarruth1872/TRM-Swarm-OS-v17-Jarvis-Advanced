//! swarm-k3 — K3 ternary collapse + voltage codec (Rust port).
//!
//! Phase 4 Step 1: port of the C++ `c_kleene_quantize_avx2` +
//! `c_voltage_encode/collapse` from libswarm_kernel_ops.so.
//!
//! Canonical K3 code table (Phase 2 unification — the SAME table as
//! TurboVec and the NAND fabric v2):
//!   FALSE(-1) → 0b00 (0)   UNKNOWN(0) → 0b01 (1)   TRUE(+1) → 0b11 (3)
//!
//! Verify gate: byte-identical collapse + quantization vs the C++ kernel.

/// Element-wise K3 collapse with epsilon thresholds (matches
/// c_kleene_quantize_avx2: v > eps → +1, v < -eps → -1, else 0).
pub fn collapse(values: &[f32], eps: f32) -> Vec<i8> {
    values
        .iter()
        .map(|&v| {
            if v > eps {
                1
            } else if v < -eps {
                -1
            } else {
                0
            }
        })
        .collect()
}

/// Counts [pass, analyze, block] — same return shape as the C kernel.
pub fn collapse_counts(values: &[f32], eps: f32) -> (i32, i32, i32) {
    let mut pass = 0;
    let mut analyze = 0;
    let mut block = 0;
    for &v in values {
        if v > eps {
            pass += 1;
        } else if v < -eps {
            block += 1;
        } else {
            analyze += 1;
        }
    }
    (pass, analyze, block)
}

/// Encode a float into a voltage waveform sample: phase = 2π·b/256,
/// amplitude = b·e^(i·phase). Returns (real, imag) as f32s.
/// Matches c_voltage_encode (b = (v * 128.0) as u8).
pub fn voltage_encode(v: f32) -> (f32, f32) {
    let b = ((v * 128.0) as i32).clamp(0, 255) as f32;
    let phase = 2.0 * std::f32::consts::PI * b / 256.0;
    let amp = b * phase.cos();
    let imag = b * phase.sin();
    (amp, imag)
}

/// Decode a voltage waveform back to a value (c_voltage_decode inverse).
pub fn voltage_decode(real: f32, imag: f32) -> f32 {
    let phase = imag.atan2(real);
    let b = phase * 256.0 / (2.0 * std::f32::consts::PI);
    (b / 128.0).clamp(-1.0, 1.0)
}

/// Pack trits to 2-bit bytes with the CANONICAL K3 table (v2).
/// FALSE(-1)→0, UNKNOWN(0)→1, TRUE(+1)→3. 4 trits per byte, LSB first.
pub fn pack_canonical(trits: &[i8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((trits.len() + 3) / 4);
    let mut acc: u8 = 0;
    let mut shift: u8 = 0;
    for &t in trits {
        let code: u8 = match t {
            1 => 3,     // TRUE
            -1 => 0,    // FALSE
            _ => 1,     // UNKNOWN
        };
        acc |= code << shift;
        shift += 2;
        if shift == 8 {
            out.push(acc);
            acc = 0;
            shift = 0;
        }
    }
    if shift != 0 {
        out.push(acc);
    }
    out
}

/// Unpack 2-bit canonical bytes back to {-1, 0, +1}.
pub fn unpack_canonical(bytes: &[u8], count: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(count);
    'outer: for &b in bytes {
        for shift in [0u8, 2, 4, 6] {
            if out.len() >= count {
                break 'outer;
            }
            out.push(match (b >> shift) & 0b11 {
                0 => -1,  // FALSE
                3 => 1,   // TRUE
                _ => 0,   // UNKNOWN (incl. unused 0b10)
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_matches_c_semantics() {
        // Reference: C kernel returns [1, 0, -1] for [0.5, 0.0, -0.5] @ eps 0.35
        assert_eq!(collapse(&[0.5, 0.0, -0.5], 0.35), vec![1, 0, -1]);
        assert_eq!(collapse_counts(&[0.5, 0.0, -0.5], 0.35), (1, 1, 1));
    }

    #[test]
    fn canonical_roundtrip_exact() {
        let trits: Vec<i8> = vec![1, 0, -1, 1, 0, -1, 1, 1, -1, 0, 1, -1, 0, 1, -1, 1];
        let packed = pack_canonical(&trits);
        assert_eq!(unpack_canonical(&packed, trits.len()), trits);
    }

    #[test]
    fn voltage_roundtrip() {
        let (r, i) = voltage_encode(0.5);
        let back = voltage_decode(r, i);
        assert!((back - 0.5).abs() < 0.02, "back={back}");
    }

    #[test]
    fn unknown_encodes_to_1() {
        assert_eq!(pack_canonical(&[0]), vec![0b01]);
        assert_eq!(unpack_canonical(&[0b01], 1), vec![0]);
    }
}
