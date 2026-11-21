//! TRM Swarm OS - K3 Ternary Kleene Logic Engine (no_std Rust port).
//! 1:1 with swarm_v2/core/ternary_kleene_engine.py:
//!   - K3 operators: AND=min, OR=max, NOT=negate, IMPLY, EQUIV
//!   - State-space reduction : R^d -> T^m (threshold bands)
//!   - Bypass decision D() = _j _j (min over trits)
//!   - Trit packing 5->1 byte, dual-bitvector (P,N) encoding
//!   - Ambiguity-based offload routing

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

// === TRIT DEFINITIONS ===
pub const FALSE: i8 = -1;
pub const UNKNOWN: i8 = 0;
pub const TRUE: i8 = 1;

/// Kleene K3 consensus over a council tally.
///
/// Kleene logic is the reason this exists: an abstention is UNKNOWN, not FALSE. The platform's
/// verdict rule compared only `approve_count` against a threshold, so 8 approve / 0 reject /
/// 6 abstain and 0 approve / 0 reject / 14 abstain both came out REJECTED -- as if the council had
/// rejected, when it had never actually said no. Here the two are distinguishable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct K3Consensus {
    /// Kleene AND of the agent trits: FALSE if anyone rejected, UNKNOWN if anyone abstained,
    /// TRUE only if every agent approved.
    pub k3_and: i8,
    /// Kleene OR: TRUE if anyone approved, UNKNOWN if anyone abstained, FALSE if all rejected.
    pub k3_or: i8,
    /// Number of abstentions (UNKNOWN trits).
    pub unknown: usize,
    /// Total votes seen.
    pub n: usize,
}

/// Consensus directly from a tally. AND/OR over a multiset depend only on the counts, so this is
/// exact rather than an approximation of the per-agent fold.
pub fn k3_from_counts(approve: usize, abstain: usize, reject: usize) -> K3Consensus {
    let n = approve + abstain + reject;
    let (k3_and, k3_or) = if n == 0 {
        (UNKNOWN, UNKNOWN)
    } else {
        (
            if reject > 0 {
                FALSE
            } else if abstain > 0 {
                UNKNOWN
            } else {
                TRUE
            },
            if approve > 0 {
                TRUE
            } else if abstain > 0 {
                UNKNOWN
            } else {
                FALSE
            },
        )
    };
    K3Consensus {
        k3_and,
        k3_or,
        unknown: abstain,
        n,
    }
}

/// Consensus by folding the individual agent trits with the Kleene operators.
/// Must agree with `k3_from_counts` for any tally (asserted by a test).
pub fn k3_consensus(trits: &[i8]) -> K3Consensus {
    if trits.is_empty() {
        return K3Consensus {
            k3_and: UNKNOWN,
            k3_or: UNKNOWN,
            unknown: 0,
            n: 0,
        };
    }
    let mut acc_and = TRUE;
    let mut acc_or = FALSE;
    let mut unknown = 0usize;
    for &t in trits {
        let t = t.clamp(FALSE, TRUE);
        if t == UNKNOWN {
            unknown += 1;
        }
        acc_and = KleeneTernaryEngine::kleene_and(acc_and, t);
        acc_or = KleeneTernaryEngine::kleene_or(acc_or, t);
    }
    K3Consensus {
        k3_and: acc_and,
        k3_or: acc_or,
        unknown,
        n: trits.len(),
    }
}

/// Default threshold bands (identical to python DEFAULT_TAU_UPPER/LOWER).
pub const DEFAULT_TAU_UPPER: f32 = 0.35;
pub const DEFAULT_TAU_LOWER: f32 = -0.35;

/// A ternary decision state with confidence and ambiguity tracking.
#[derive(Debug, Clone, Copy)]
pub struct KleeneState {
    pub trit: i8,               // -1, 0, +1
    pub confidence: f32,        // 0.0 to 1.0
    pub ambiguity_ratio: f32,   // fraction of unknown trits
}

impl KleeneState {
    /// G6: DECLARED, NEVER CONSULTED. No gate in the vote path asks this today — the bypass
    /// decision is made by `council_ssm_pipeline::fuse` and the asymmetric veto lanes. Marked
    /// instead of deleted so the missing wiring stays visible rather than looking like an absent
    /// capability.
    #[allow(dead_code)]
    #[inline]
    pub fn is_bypassable(&self) -> bool {
        self.trit == TRUE && self.confidence >= 0.88
    }

    /// G6: declared, never consulted — see `is_bypassable`.
    #[allow(dead_code)]
    #[inline]
    pub fn is_rejected(&self) -> bool {
        self.trit == FALSE
    }
}

pub struct KleeneTernaryEngine {
    pub tau_upper: f32,
    pub tau_lower: f32,
    pub operation_count: u64,
    pub bypass_count: u64,
}

impl KleeneTernaryEngine {
    pub fn new() -> Self {
        KleeneTernaryEngine {
            tau_upper: DEFAULT_TAU_UPPER,
            tau_lower: DEFAULT_TAU_LOWER,
            operation_count: 0,
            bypass_count: 0,
        }
    }

    // === 2.1 KLEENE K3 OPERATORS ===
    #[inline]
    pub fn kleene_and(x: i8, y: i8) -> i8 {
        x.min(y)
    }

    #[inline]
    pub fn kleene_or(x: i8, y: i8) -> i8 {
        x.max(y)
    }

    #[inline]
    pub fn kleene_not(x: i8) -> i8 {
        -x
    }

    #[inline]
    pub fn kleene_imply(x: i8, y: i8) -> i8 {
        (-x).max(y)
    }

    #[inline]
    pub fn kleene_equiv(x: i8, y: i8) -> i8 {
        ((-x).max(y)).min((-y).max(x))
    }

    // === 2.2 STATE-SPACE REDUCTION : R^d -> T^m ===
    /// Quick reduction without projection matrix - threshold each value directly.
    pub fn quick_reduction(&self, values: &[f32]) -> Vec<i8> {
        values
            .iter()
            .map(|&v| {
                if v > self.tau_upper {
                    TRUE
                } else if v < self.tau_lower {
                    FALSE
                } else {
                    UNKNOWN
                }
            })
            .collect()
    }

    /// (S)_i = +1 if S, W_i + b_i > _upper; -1 if < _lower; else 0.
    /// With identity projection when W is None (same as python).
    pub fn state_reduction(&self, vector: &[f32], w: Option<&[&[f32]]>, b: Option<&[f32]>) -> Vec<i8> {
        let m = match w {
            Some(rows) => rows.len(),
            None => vector.len(),
        };
        let mut result = Vec::with_capacity(m);
        for i in 0..m {
            let projection = match w {
                Some(rows) => {
                    let row = rows[i];
                    let mut acc = 0.0f32;
                    for (s, &wv) in vector.iter().zip(row.iter()) {
                        acc += s * wv;
                    }
                    acc
                }
                None => vector[i],
            };
            let bias = match b {
                Some(bv) => bv[i],
                None => 0.0,
            };
            let proj = projection + bias;
            if proj > self.tau_upper {
                result.push(TRUE);
            } else if proj < self.tau_lower {
                result.push(FALSE);
            } else {
                result.push(UNKNOWN);
            }
        }
        result
    }

    // === 2.3 BYPASS DECISION D() = _j _j ===
    pub fn bypass_decision(&mut self, psi: &[i8]) -> KleeneState {
        let decision_trit = psi.iter().copied().min().unwrap_or(UNKNOWN);

        let total = psi.len() as f32;
        let unknown_count = psi.iter().filter(|&&t| t == UNKNOWN).count() as f32;
        let true_count = psi.iter().filter(|&&t| t == TRUE).count() as f32;
        let false_count = psi.iter().filter(|&&t| t == FALSE).count() as f32;

        let ambiguity_ratio = if total > 0.0 { unknown_count / total } else { 0.0 };

        let confidence = if decision_trit == TRUE {
            true_count / total.max(1.0)
        } else if decision_trit == FALSE {
            false_count / total.max(1.0)
        } else {
            1.0 - ambiguity_ratio
        };

        self.operation_count += 1;
        if decision_trit == TRUE && confidence >= 0.88 {
            self.bypass_count += 1;
        }

        KleeneState {
            trit: decision_trit,
            confidence,
            ambiguity_ratio,
        }
    }

    // === VECTORIZED KLEENE OPERATIONS ===
    pub fn vectorized_and(a: &[i8], b: &[i8]) -> Vec<i8> {
        a.iter().zip(b.iter()).map(|(&x, &y)| x.min(y)).collect()
    }

    pub fn vectorized_or(a: &[i8], b: &[i8]) -> Vec<i8> {
        a.iter().zip(b.iter()).map(|(&x, &y)| x.max(y)).collect()
    }

    pub fn vectorized_not(a: &[i8]) -> Vec<i8> {
        a.iter().map(|&x| -x).collect()
    }

    // === TRIT PACKING / UNPACKING ===
    /// Encode 5 trits {-1,0,+1} -> {0,1,2} into one byte (0-242).
    pub fn pack_5_trits(t: [i8; 5]) -> u8 {
        let mut byte: u32 = 0;
        for (i, &trit) in t.iter().enumerate() {
            let v = (trit + 1) as u32;
            byte += v * 3u32.pow(i as u32);
        }
        byte as u8
    }

    /// Decode one byte (0-242) back into 5 trits {-1, 0, +1}.
    pub fn unpack_5_trits(byte: u8) -> [i8; 5] {
        let mut out = [0i8; 5];
        let mut remaining = byte as u32;
        for out_trit in out.iter_mut() {
            let value = (remaining % 3) as i8;
            *out_trit = value - 1;
            remaining /= 3;
        }
        out
    }

    // === DUAL-BITVECTOR REPRESENTATION ===
    pub fn trit_vector_to_bitvectors(psi: &[i8]) -> (Vec<u8>, Vec<u8>) {
        let p: Vec<u8> = psi.iter().map(|&t| if t == TRUE { 1 } else { 0 }).collect();
        let n: Vec<u8> = psi.iter().map(|&t| if t == FALSE { 1 } else { 0 }).collect();
        (p, n)
    }

    pub fn bitvectors_to_trit(p: &[u8], n: &[u8]) -> Vec<i8> {
        p.iter()
            .zip(n.iter())
            .map(|(&p, &n)| {
                if p == 1 {
                    TRUE
                } else if n == 1 {
                    FALSE
                } else {
                    UNKNOWN
                }
            })
            .collect()
    }

    // === AMBIGUITY-BASED OFFLOAD DECISION ===
    /// ratio == 0.0 -> NATIVE_BYPASS; <= 0.60 -> INSTELLA_MOE; else BONSAI_TERNARY
    pub fn evaluate_offload(&self, psi: &[i8]) -> &'static str {
        let total = psi.len() as f32;
        let unknown_count = psi.iter().filter(|&&t| t == UNKNOWN).count() as f32;
        let ratio = if total > 0.0 { unknown_count / total } else { 0.0 };

        if ratio == 0.0 {
            "NATIVE_BYPASS"
        } else if ratio <= 0.60 {
            "INSTELLA_MOE"
        } else {
            "BONSAI_TERNARY"
        }
    }

    // === STATISTICS ===
    pub fn bypass_rate(&self) -> f32 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.bypass_count as f32 / self.operation_count as f32
        }
    }
}

/// Synthesize a continuous state vector from a task hash (32 bytes -> [-1,+1]).
/// Mirrors `intercept_with_kleene`'s sha256 -> float vector (FIPS 180-4 sha256).
pub fn task_hash_vector(task: &[u8]) -> [f32; 32] {
    let mut out = [0.0f32; 32];
    let h = sha256(task);
    for (i, &b) in h.iter().enumerate() {
        out[i] = (b as f32 / 255.0) * 2.0 - 1.0;
    }
    out
}

/// Minimal FIPS 180-4 SHA-256 (no_std) - used for task-hash feature extraction.
pub fn sha256(data: &[u8]) -> [u8; 32] {
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

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, &word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}
