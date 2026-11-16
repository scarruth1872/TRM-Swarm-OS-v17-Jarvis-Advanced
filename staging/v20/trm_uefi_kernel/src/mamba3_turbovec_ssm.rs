//! TRM Swarm OS — Mamba-3 Linear State Space Model (SSM) & TurboVec SIMD Quantization Engine (no_std port).
//! Mathematical Framework:
//!   State Recurrence: h_t = exp(Δ A) h_{t-1} + Δ B x_t,  y_t = C h_t
//!   TurboVec SIMD: 4-bit scalar quantization + exact cosine similarity
//!   Complexity: O(1) memory per step vs O(N^2) Transformer KV-cache
//! 1:1 deterministic core from swarm_v2/core/mamba3_turbovec_ssm.py.
//!
//! PARITY NOTES (verified against the Python oracle, numpy 2.4.3, float32 path):
//!  * The oracle's whole numeric path is float32 (numpy NEP-50 weak scalars), so the
//!    port is float32 throughout — not a promotion to f64.
//!  * `Mamba3StateSpaceScanner::step` previously used the 2nd-order Taylor series
//!    1 + d + d²/2 for A_bar; the oracle uses a true exp(). At the engine's delta
//!    (0.02) that approximation was off by up to 6.6e-4 absolute / 7.7e-4 relative
//!    per state slot (~11 000 f32 ulp) and 3.4e-4 relative in y. Replaced with a
//!    hand-rolled exp (see below).
//!  * HOST-SIDE (deliberately not ported): the ctypes CDLL load of
//!    strip_mamba3_turbovec_ssm.dll and the `_NATIVE_LOADED` / `_NATIVE_REASON`
//!    bookkeeping, `_os.path` resolution, and the `time.perf_counter()` latency
//!    measurements in `process_query`.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const D_MODEL: usize = 128;
pub const D_STATE: usize = 16;

// =============================================================================
// Hand-rolled math (no_std: core has no f32::exp / f32::sqrt / f32::round /
// f32::floor — those live in std, which the UEFI kernel does not link).  Every
// routine below is built from primitive f32/f64 +,-,*,/ and bit casts only.
// =============================================================================

/// floor(x) without std.
#[inline]
pub fn f_floor(x: f32) -> f32 {
    if x != x || x == f32::INFINITY || x == f32::NEG_INFINITY || x == 0.0 {
        return x;
    }
    // Every f32 with |x| >= 2^23 is already an integer, and the i64 cast below
    // would saturate (and floor(1e38) must stay 1e38, not 9.22e18).
    if x >= 8388608.0 || x <= -8388608.0 {
        return x;
    }
    let t = x as i64 as f32; // truncate toward zero (Rust float->int casts saturate)
    if t > x {
        t - 1.0
    } else {
        t
    }
}

/// numpy's `np.round`: round half to EVEN (banker's rounding), unlike Rust's
/// absent std `f32::round` which rounds half away from zero. The oracle's
/// `TurboVecQuantizer::quantize` calls `np.round`, and exact .5 inputs ARE
/// reachable (113 of 320e6 sampled quantizer inputs landed on an exact .5,
/// 66 of them changing the code under half-even vs half-away).
#[inline]
pub fn f_round_half_even(x: f32) -> f32 {
    let fl = f_floor(x);
    let d = x - fl; // exact in f32 for finite x
    let r = if d > 0.5 {
        fl + 1.0
    } else if d < 0.5 {
        fl
    } else {
        let half = fl * 0.5;
        if (half - f_floor(half)) == 0.0 {
            fl // fl is even -> keep it
        } else {
            fl + 1.0
        }
    };
    // np.round keeps the sign of a zero result (np.round(-0.3) == -0.0)
    if r == 0.0 {
        0.0f32.copysign(x)
    } else {
        r
    }
}

/// sqrt(f64) by bit-halving seed + Newton iterations (quadratic convergence:
/// ~1e-2 -> ~1e-16 in 6 steps, i.e. correctly rounded to the last bit).
pub fn f64_sqrt(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x != x || x == f64::INFINITY {
        return x;
    }
    let mut y = f64::from_bits((x.to_bits() >> 1) + 0x1ff8_0000_0000_0000);
    let mut i = 0;
    while i < 6 {
        y = 0.5 * (y + x / y);
        i += 1;
    }
    y
}

/// sqrt(f32): the operand is exact in f64, so this is the correctly rounded f32
/// root (double rounding on sqrt of an exact f32 is a 2^-53 tie case).
#[inline]
pub fn f32_sqrt(x: f32) -> f32 {
    f64_sqrt(x as f64) as f32
}

/// exp(f64) = 2^k * exp(r), r = x - k*ln2 (two-part ln2), |r| <= 0.3466,
/// Taylor to r^13 (truncation error < 4e-18, far below the f32 target).
pub fn f64_exp(x: f64) -> f64 {
    if x != x {
        return x;
    }
    if x > 709.782_712_893_384 {
        return f64::INFINITY;
    }
    if x < -745.133_219_101_941_1 {
        return 0.0;
    }
    const INV_LN2: f64 = 1.442_695_040_888_963_4;
    const LN2_HI: f64 = 6.931_471_803_691_238_164_90e-1;
    const LN2_LO: f64 = 1.908_214_929_270_587_700_02e-10;
    let kf = x * INV_LN2;
    // round-to-nearest integer via the 2^52 add/subtract trick (no std::round)
    let k = if kf >= 0.0 {
        (kf + 4503599627370496.0) - 4503599627370496.0
    } else {
        (kf - 4503599627370496.0) + 4503599627370496.0
    };
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let p = 1.0
        + r * (1.0
            + r * (0.5
                + r * (1.666_666_666_666_666_57e-01
                    + r * (4.166_666_666_666_666_44e-02
                        + r * (8.333_333_333_333_333_22e-03
                            + r * (1.388_888_888_888_888_94e-03
                                + r * (1.984_126_984_126_984_13e-04
                                    + r * (2.480_158_730_158_730_16e-05
                                        + r * (2.755_731_922_398_589_25e-06
                                            + r * (2.755_731_922_398_588_89e-07
                                                + r * (2.505_210_838_544_172_02e-08
                                                    + r * (2.087_675_698_786_809_90e-09
                                                        + r * 1.605_904_383_682_161_30e-10))))))))))));
    ldexp2(p, k as i32)
}

/// multiply by 2^e without std::ldexp.
#[inline]
fn ldexp2(m: f64, mut e: i32) -> f64 {
    let mut r = m;
    while e > 1023 {
        r *= f64::from_bits(2046u64 << 52); // 2^1023
        e -= 1023;
    }
    while e < -1022 {
        r *= f64::from_bits(1u64 << 52); // 2^-1022
        e += 1022;
    }
    r * f64::from_bits(((e + 1023) as u64) << 52)
}

/// exp(f32): oracle is `np.exp` on a float32 operand; computed in f64 and rounded
/// once, so the result is within 1 ulp of the correctly rounded f32 exp.
#[inline]
pub fn f32_exp(x: f32) -> f32 {
    f64_exp(x as f64) as f32
}

// =============================================================================
// CPython string hashing — the oracle's `Mamba3TurboVecEngine::process_query`
// seeds its query vector from `hash(word) % 1000`.  CPython's str hash is
// siphash13 over the string's *internal* representation with a per-process
// random key; with PYTHONHASHSEED=0 (the only deterministic setting) the key is
// zero, and the representation is 1/2/4 bytes per char (latin-1 / UCS-2LE /
// UCS-4LE) depending on the largest code point.  Both rules are reproduced here.
// =============================================================================

#[inline]
fn sipround(v: &mut [u64; 4]) {
    v[0] = v[0].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(13);
    v[1] ^= v[0];
    v[0] = v[0].rotate_left(32);
    v[2] = v[2].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(16);
    v[3] ^= v[2];
    v[0] = v[0].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(21);
    v[3] ^= v[0];
    v[2] = v[2].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(17);
    v[1] ^= v[2];
    v[2] = v[2].rotate_left(32);
}

/// siphash13 (the CPython variant: 1 compression round, 3 finalization rounds).
pub fn siphash13_64(k0: u64, k1: u64, data: &[u8]) -> u64 {
    let mut v = [
        k0 ^ 0x736f_6d65_7073_6575,
        k1 ^ 0x646f_7261_6e64_6f6d,
        k0 ^ 0x6c79_6765_6e65_7261,
        k1 ^ 0x7465_6462_7974_6573,
    ];
    let n = data.len();
    let mut i = 0;
    while n - i >= 8 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&data[i..i + 8]);
        let m = u64::from_le_bytes(buf);
        v[3] ^= m;
        sipround(&mut v);
        v[0] ^= m;
        i += 8;
    }
    let mut b: u64 = ((n as u64) & 0xFF) << 56;
    let mut j = 0usize;
    while i + j < n {
        b |= (data[i + j] as u64) << (8 * j);
        j += 1;
    }
    v[3] ^= b;
    sipround(&mut v);
    v[0] ^= b;
    v[2] ^= 0xFF;
    let mut r = 0;
    while r < 3 {
        sipround(&mut v);
        r += 1;
    }
    v[0] ^ v[1] ^ v[2] ^ v[3]
}

/// CPython `hash(str)` with the zero hash key (PYTHONHASHSEED=0).
pub fn py_str_hash(s: &str) -> i64 {
    if s.is_empty() {
        return 0; // CPython: hash("") == 0
    }
    // kind = bytes per char in CPython's internal representation
    let mut kind = 1usize;
    for c in s.chars() {
        let cp = c as u32;
        if cp >= 0x10000 {
            kind = 4;
            break;
        } else if cp >= 0x100 {
            kind = 2;
        }
    }
    let mut data: Vec<u8> = Vec::with_capacity(s.len() * kind);
    for c in s.chars() {
        let cp = c as u32;
        let mut b = 0usize;
        while b < kind {
            data.push(((cp >> (8 * b)) & 0xFF) as u8);
            b += 1;
        }
    }
    let x = siphash13_64(0, 0, &data) as i64;
    if x == -1 {
        -2
    } else {
        x
    }
}

/// Python `int(hash(word) % 1000)` — Python's % on a negative int is non-negative.
#[inline]
pub fn py_hash_mod1000(word: &str) -> u32 {
    py_str_hash(word).rem_euclid(1000) as u32
}

// =============================================================================
// Mamba-3 SSM
// =============================================================================

pub struct Mamba3StateSpaceScanner {
    pub h: [[f32; D_MODEL]; D_STATE],
    pub a_diag: [f32; D_STATE],
    pub step_count: u64,
}

impl Mamba3StateSpaceScanner {
    pub fn new() -> Self {
        let mut a_diag = [0.0f32; D_STATE];
        for i in 0..D_STATE {
            a_diag[i] = -0.5 * ((i + 1) as f32);
        }
        Mamba3StateSpaceScanner {
            h: [[0.0f32; D_MODEL]; D_STATE],
            a_diag,
            step_count: 0,
        }
    }

    /// Discretized step transition (oracle: Mamba3StateSpaceScanner.step):
    ///   A_bar = exp(delta * A_diag)        (float32, np.exp)
    ///   B_bar = delta * B = delta          (B is all-ones)
    ///   h_t   = A_bar[:,None] * h_{t-1} + B_bar * x
    ///   y_t   = sum_s (1/sqrt(D_STATE)) * h_t[s]
    /// Returns output vector y_t (128-dim) in constant O(1) memory.
    /// `x` shorter than D_MODEL is zero-padded (numpy would broadcast-error).
    pub fn step(&mut self, x: &[f32], delta: f32) -> [f32; D_MODEL] {
        let mut y = [0.0f32; D_MODEL];
        self.step_count += 1;

        for s in 0..D_STATE {
            // A_bar = exp(delta * a_diag[s]) — hand-rolled exp, see f32_exp.
            // (was: 1 + da + da*da*0.5, a Taylor approximation that deviated from
            //  the oracle by up to 6.6e-4 at delta = 0.02.)
            let da = delta * self.a_diag[s];
            let a_bar = f32_exp(da);
            let b_bar = delta;

            let c_val = 1.0 / 4.0; // 1 / sqrt(16)

            for m in 0..D_MODEL {
                let in_val = if m < x.len() { x[m] } else { 0.0 };
                let h_next = a_bar * self.h[s][m] + b_bar * in_val;
                self.h[s][m] = h_next;
                y[m] += c_val * h_next;
            }
        }

        y
    }

    pub fn reset_state(&mut self) {
        for s in 0..D_STATE {
            for m in 0..D_MODEL {
                self.h[s][m] = 0.0;
            }
        }
    }
}

// =============================================================================
// TurboVec quantizer (oracle: class TurboVecQuantizer)
// =============================================================================

pub struct TurboVecQuantizer {
    pub dim: usize,
    pub bit_width: u32,
    pub max_val: u32,
}

impl TurboVecQuantizer {
    pub fn new(dim: usize, bit_width: u32) -> Self {
        TurboVecQuantizer {
            dim,
            bit_width,
            max_val: (1u32 << bit_width) - 1, // 15 for 4-bit
        }
    }

    /// oracle: `normalize` — L2 normalisation along the last axis with the
    /// zero-norm guard (norm == 0 -> divide by 1.0). Sum of squares is
    /// accumulated in f32 exactly like numpy's `np.dot(x, x)`, so subnormal
    /// vectors whose squares underflow to 0.0 take the guard here too.
    pub fn normalize(&self, v: &[f32]) -> Vec<f32> {
        let mut sq = 0.0f32;
        for x in v {
            sq += *x * *x;
        }
        let mut norm = f32_sqrt(sq);
        if norm == 0.0 {
            norm = 1.0;
        }
        let mut out = Vec::with_capacity(v.len());
        for x in v {
            out.push(*x / norm);
        }
        out
    }

    /// oracle: `quantize` -> (q_codes, scale, v_min).
    /// `q = np.round((v_norm - v_min) / scale * max_val)` with half-to-even
    /// rounding and a uint8 cast. `scale` falls back to 1.0 when the normalised
    /// range is <= 1e-8 (the threshold compare is done on the exact f64 range,
    /// as the oracle holds it as a Python float).
    pub fn quantize(&self, v: &[f32]) -> (Vec<u8>, f32, f32) {
        if v.is_empty() {
            // oracle raises ValueError("zero-size array to reduction ...")
            return (Vec::new(), 1.0, 0.0);
        }
        let v_norm = self.normalize(v);
        let mut v_min = f32::INFINITY;
        let mut v_max = f32::NEG_INFINITY;
        for x in &v_norm {
            if *x < v_min {
                v_min = *x;
            }
            if *x > v_max {
                v_max = *x;
            }
        }
        let range = (v_max as f64) - (v_min as f64);
        let scale = if range > 1e-8 { range as f32 } else { 1.0f32 };
        let mx = self.max_val as f32;
        let mut codes = Vec::with_capacity(v_norm.len());
        for x in &v_norm {
            let r = ((*x - v_min) / scale) * mx;
            let q = f_round_half_even(r);
            codes.push(if q <= 0.0 {
                0u8
            } else if q >= 255.0 {
                255u8
            } else {
                q as u8
            });
        }
        (codes, scale, v_min)
    }

    /// oracle: `dequantize` — (q / max_val) * scale + v_min, all in f32.
    pub fn dequantize(&self, q_codes: &[u8], scale: f32, v_min: f32) -> Vec<f32> {
        let mx = self.max_val as f32;
        let mut out = Vec::with_capacity(q_codes.len());
        for q in q_codes {
            out.push(((*q as f32) / mx) * scale + v_min);
        }
        out
    }

    /// oracle: `fast_cosine_similarity` — exact normalised inner product
    /// `db_norm . q_norm` (np.dot), one score per row of the flattened
    /// row-major (n_rows, dim) database matrix.
    pub fn fast_cosine_similarity(&self, q: &[f32], db_matrix: &[f32]) -> Vec<f32> {
        if self.dim == 0 {
            return Vec::new();
        }
        let q_norm = self.normalize(q);
        let rows = db_matrix.len() / self.dim;
        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let row = &db_matrix[r * self.dim..(r + 1) * self.dim];
            let db_norm = self.normalize(row);
            let n = self.dim.min(q_norm.len());
            let mut acc = 0.0f32;
            for i in 0..n {
                acc += db_norm[i] * q_norm[i];
            }
            out.push(acc);
        }
        out
    }
}

// =============================================================================
// Unified engine (oracle: class Mamba3TurboVecEngine)
// =============================================================================

/// Mirrors the dict returned by the oracle's `process_query`. Latencies are
/// host-side wall-clock measurements in both languages and are not reproducible.
pub struct TurboVecQueryResult {
    pub top1_expert_id: usize,
    pub top1_score: f32,
    pub all_scores: Vec<f32>,
    pub ssm_latency_ms: f32,
    pub simd_latency_ms: f32,
    pub total_engine_latency_ms: f32,
    pub complexity: &'static str,
}

pub struct Mamba3TurboVecEngine {
    pub ssm: Mamba3StateSpaceScanner,
    pub quantizer: TurboVecQuantizer,
    pub d_model: usize,
}

impl Mamba3TurboVecEngine {
    pub fn new() -> Self {
        Mamba3TurboVecEngine {
            ssm: Mamba3StateSpaceScanner::new(),
            quantizer: TurboVecQuantizer::new(D_MODEL, 4),
            d_model: D_MODEL,
        }
    }

    pub fn process_step(&mut self, x: &[f32]) -> ([f32; D_MODEL], u64) {
        let t0 = crate::time::now_ns();
        let y = self.ssm.step(x, 0.01);
        let elapsed_ns = crate::time::now_ns().saturating_sub(t0);
        (y, elapsed_ns)
    }

    /// oracle: `process_query(query_text, expert_embeddings)`.
    /// `expert_embeddings` is the flattened row-major (n_experts, d_model) matrix.
    /// delta = 0.02 and the word hash rules are the oracle's; see py_str_hash.
    /// np.argmax semantics: first maximal index (ties keep the lower index), and
    /// an empty score vector yields top1_expert_id = 0 (numpy raises).
    pub fn process_query(&mut self, query_text: &str, expert_embeddings: &[f32]) -> TurboVecQueryResult {
        let t0 = crate::time::now_ns();

        // 1. Deterministic hashing of query_text to float32 vector
        let mut raw_vec = [0.0f32; D_MODEL];
        let lower = query_text.to_lowercase();
        let mut idx = 0usize;
        for word in lower.split_whitespace() {
            let val = (py_hash_mod1000(word) as f32) / 1000.0;
            raw_vec[idx % self.d_model] += val;
            idx += 1;
        }

        // 2. Mamba-3 SSM step linear transformation (delta = 0.02)
        let t_ssm_start = crate::time::now_ns();
        let transformed_vec = self.ssm.step(&raw_vec, 0.02);
        let ssm_latency_ms = (crate::time::now_ns().saturating_sub(t_ssm_start)) as f32 / 1.0e6;

        // 3. TurboVec fast SIMD cosine distance search across experts
        let t_simd_start = crate::time::now_ns();
        let sim_scores = self
            .quantizer
            .fast_cosine_similarity(&transformed_vec, expert_embeddings);
        let mut top1_expert_id = 0usize;
        let mut top1_score = f32::NEG_INFINITY;
        for (i, s) in sim_scores.iter().enumerate() {
            if *s > top1_score {
                top1_score = *s;
                top1_expert_id = i;
            }
        }
        if sim_scores.is_empty() {
            top1_score = f32::NEG_INFINITY;
        }
        let simd_latency_ms = (crate::time::now_ns().saturating_sub(t_simd_start)) as f32 / 1.0e6;

        let total_engine_latency_ms = (crate::time::now_ns().saturating_sub(t0)) as f32 / 1.0e6;

        TurboVecQueryResult {
            top1_expert_id,
            top1_score,
            all_scores: sim_scores,
            ssm_latency_ms,
            simd_latency_ms,
            total_engine_latency_ms,
            complexity: "O(1) Memory Constant Space / O(D) SIMD Compute",
        }
    }
}

static SINGLETON_MAMBA3_ENGINE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_mamba3_engine() -> u64 {
    let v = SINGLETON_MAMBA3_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MAMBA3_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
