//! TRM Swarm OS — SRAM Pattern Computer (no_std port).
//! 1:1 core semantics from swarm_v2/core/sram_pattern_computer.py — Catalyst.
//! L3 SRAM pattern engines: 15 Virtual Architectures spanning physical,
//! mathematical, and sacred geometric computing fabrics. Real L3 constants.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// L3 SRAM constants (1:1 python).
pub const L3_SIZE: u64 = 32 * 1024 * 1024; // 32 MB
pub const L3_SLICES: u64 = 4;
pub const L3_LINE_SIZE: u64 = 64;
pub const L3_WAYS: u64 = 16;
pub const L3_TRANSISTORS_PER_CELL: u64 = 6;
pub const L3_TOTAL_CELLS: u64 = L3_SIZE * 8; // 256M cells
pub const L3_TOTAL_TRANSISTORS: u64 = L3_TOTAL_CELLS * L3_TRANSISTORS_PER_CELL; // 1.5B+ transistors

/// 1. Pattern shift (FPGA-style LUT transform).
pub fn sram_lut_fpga(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for (i, &b) in input.iter().enumerate() {
        let r = (i as u64 % L3_WAYS) as u8;
        let rotated = b.rotate_left(r as u32);
        out.push(rotated ^ ((L3_SLICES as u8) + r));
    }
    out
}

/// 2. Cellular automaton pattern (elementary rule 30-ish over the line).
pub fn sram_lut_cellular_automaton(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let mut out = vec![0u8; n];
    for i in 0..n {
        let left = input[(i + n - 1) % n];
        let center = input[i];
        let right = input[(i + 1) % n];
        let a = (left >> 7) & 1;
        let b = (center >> 7) & 1;
        let c = (right >> 7) & 1;
        let rule30 = (a ^ (b | c)) as u8;
        out[i] = (center << 1) | rule30;
    }
    out
}

/// 3. Neural pattern: activation thresholding over cache line.
pub fn sram_lut_neural_network(weights: &[i8], inputs: &[i8]) -> Vec<i8> {
    let n = weights.len().min(inputs.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let prod = (weights[i] as i32 * inputs[i] as i32) / 128;
        out.push(prod.clamp(-128, 127) as i8);
    }
    out
}

/// 4. Content-addressable pattern match: count lines matching a key.
pub fn sram_lut_content_addressable(rows: &[&[u8]], key: &[u8]) -> u64 {
    rows.iter()
        .filter(|r| r.len() == key.len() && r.iter().zip(key.iter()).all(|(a, b)| a == b))
        .count() as u64
}

/// 5. Quantum-simulator pattern: superposition of two patterns (XOR mean).
pub fn sram_lut_quantum_simulator(a: &[u8], b: &[u8]) -> Vec<u8> {
    let n = a.len().min(b.len());
    (0..n).map(|i| (a[i] ^ b[i]) / 2).collect()
}

/// 6. Ising Spin Glass: simulated annealing spin update s_i = sign(sum(J_ij * s_j) + h_i).
pub fn sram_lut_ising_spin_glass(spins: &[i8], couplings: &[i8]) -> Vec<i8> {
    let n = spins.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut field: i32 = 0;
        for j in 0..n {
            let j_idx = (i * n + j) % couplings.len();
            field += couplings[j_idx] as i32 * spins[j] as i32;
        }
        out.push(if field >= 0 { 1 } else { -1 });
    }
    out
}

/// 7. Hopfield Attractor Network: recall stored associative memories.
pub fn sram_lut_hopfield_recall(state: &[i8], weights: &[i8], dim: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        let mut sum: i32 = 0;
        for j in 0..dim {
            let w_idx = (i * dim + j) % weights.len();
            sum += weights[w_idx] as i32 * state[j % state.len()] as i32;
        }
        out.push(if sum >= 0 { 1 } else { -1 });
    }
    out
}

/// 8. Boltzmann Machine: stochastic energy sampling E = -sum(w_ij * s_i * s_j).
pub fn sram_lut_boltzmann_energy(spins: &[i8], weights: &[i8], dim: usize) -> i64 {
    let mut energy: i64 = 0;
    for i in 0..dim {
        for j in 0..dim {
            let w_idx = (i * dim + j) % weights.len();
            energy -= (weights[w_idx] as i64) * (spins[i % spins.len()] as i64) * (spins[j % spins.len()] as i64);
        }
    }
    energy
}

/// 9. GNN Accelerator: message passing aggregation over graph nodes.
pub fn sram_lut_gnn_aggregate(node_features: &[i32], adj_matrix: &[u8], num_nodes: usize) -> Vec<i32> {
    let mut out = vec![0i32; num_nodes];
    for i in 0..num_nodes {
        let mut agg = node_features[i % node_features.len()];
        for j in 0..num_nodes {
            if adj_matrix[(i * num_nodes + j) % adj_matrix.len()] != 0 {
                agg += node_features[j % node_features.len()];
            }
        }
        out[i] = agg;
    }
    out
}

/// 10. Ternary CAM (TCAM): K3 balanced wildcard matching (+1, 0, -1).
pub fn sram_lut_tcam_match(stored_trits: &[i8], query_trits: &[i8], mask: &[u8]) -> bool {
    let n = stored_trits.len().min(query_trits.len());
    for i in 0..n {
        let is_wildcard = mask.get(i).copied().unwrap_or(0) == 0;
        if !is_wildcard && stored_trits[i] != query_trits[i] {
            return false;
        }
    }
    true
}

/// 11. Systolic Array: 2D matrix multiplication pulse.
pub fn sram_lut_systolic_matmul(a: &[i32], b: &[i32], dim: usize) -> Vec<i32> {
    let mut c = vec![0i32; dim * dim];
    for i in 0..dim {
        for k in 0..dim {
            for j in 0..dim {
                c[i * dim + j] += a[(i * dim + k) % a.len()] * b[(k * dim + j) % b.len()];
            }
        }
    }
    c
}

/// 12. Sparse Attention: top-k sparsity filter.
pub fn sram_lut_sparse_attention(scores: &[i32], top_k: usize) -> Vec<i32> {
    let mut out = scores.to_vec();
    if top_k >= out.len() {
        return out;
    }
    out.sort_unstable_by(|a, b| b.cmp(a));
    out.truncate(top_k);
    out
}

/// 13. Hyperdimensional Computing (HDC): high-dimensional binding (XOR) & bundling (majority).
pub fn sram_lut_hdc_bind(hv_a: &[u8], hv_b: &[u8]) -> Vec<u8> {
    hv_a.iter().zip(hv_b.iter()).map(|(&a, &b)| a ^ b).collect()
}

/// 14. Holographic Associative Memory: the circular convolution — the Plate HRR bind. The earlier
/// `a/2 + b/2` average was a stub (a midpoint, not a decodable association). The circular convolution is
/// the real holographic bind: `c[k] = Σ_j a[j]·b[(k−j) mod n]`, a vector unlike both operands that the
/// circular correlation can unbind to recover either one, and that superposes (memory = Σ of bound pairs).
pub fn sram_lut_holographic_interference(a: &[i16], b: &[i16]) -> Vec<i32> {
    let n = a.len().min(b.len());
    if n == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let mut acc: i64 = 0;
        for j in 0..n {
            acc += (a[j] as i64) * (b[((k + n - j) % n)] as i64);
        }
        // i32 output: the convolution of the Gaussian (scaled) operands reaches ~1e8, which overflows
        // i16 (the earlier clamp destroyed the association); i32 holds it, keeping the bind decodable.
        out.push(acc as i32);
    }
    out
}

/// 15. Oscillatory Neural Network (ONN): Kuramoto phase synchronization bound to 396-963 Hz.
pub fn sram_lut_onn_sync(phases: &[i32], coupling_k: i32) -> Vec<i32> {
    let n = phases.len();
    let mut new_phases = Vec::with_capacity(n);
    for i in 0..n {
        let mut d_theta: i32 = 0;
        for j in 0..n {
            let diff = phases[j] - phases[i];
            d_theta += (diff * coupling_k) / 100;
        }
        new_phases.push((phases[i] + d_theta) % 360);
    }
    new_phases
}

/// L3 capacity: slices * ways * line_size * transistors.
pub fn l3_capacity_bits() -> u64 {
    L3_SLICES * L3_WAYS * L3_LINE_SIZE * 8
}

// ═══════════════════════════════════════════════════════════════════════════
// PORTED CORE — 1:1 with swarm_v2/core/sram_pattern_computer.py (the oracle).
//
// The 15 `pattern_*` functions above this line are byte-transform helpers with
// no counterpart in the python module (invented; e.g. python's `sram_lut_fpga`
// is a config factory, not a LUT transform).  They are left untouched.  What
// follows is the oracle's own portable content, appended under the oracle's
// own names:
//   * L3 geometry (L3_SETS) and the ARCHITECTURES key order,
//   * the 15 pattern config factories (`pattern_*` builders + `load_architecture`),
//   * `summary`, the native strip rule (`strip_batch_sram_pattern_reconfig`),
//   * the compute kernels (`compute_*`, `calc_h`, `calc_free_energy`),
//   * `execute_computational_algorithm` / `reconfigure_and_execute`.
//
// Not ported, because host-side: wall-clock `latency_us` and the
// `benchmark_all` / `benchmark_all_native` timing loops, the ctypes CDYLIB /
// strip loader, `hashlib` usage beyond the ported SHA-256, and the
// process-wide `get_sram_computer()` singleton (no mutable statics in a
// no_std UEFI kernel).
//
// The numpy `np.random.RandomState(seed)` default draws (MT19937 + numpy's
// `choice`/`randn` transforms) are host-side too: those kernels take the drawn
// values as explicit inputs instead.  Kernels whose python API accepts explicit
// inputs (`probe_pattern`, `entries`, `A`, `Q/K/V`, `adj/X/W`, `v0`, `W`) take
// exactly those inputs, so the algorithm itself is ported 1:1.
//
// no_std: std is unavailable, so every std-only float routine used here
// (sqrt / exp / ln / log2 / sin / cos / floor / ceil / trunc / round) is
// hand-rolled below and verified against python's `math` module.
// ═══════════════════════════════════════════════════════════════════════════

// ---------------------------------------------------------------------------
// L3 cache geometry — python L3_SIZE/L3_SLICES/L3_LINE_SIZE/L3_WAYS/L3_SETS.
// ---------------------------------------------------------------------------
/// python: L3_SIZE // (L3_LINE_SIZE * L3_WAYS * L3_SLICES) == 8192 sets/slice.
pub const L3_SETS: u64 = L3_SIZE / (L3_LINE_SIZE * L3_WAYS * L3_SLICES);

// ---------------------------------------------------------------------------
// Hand-rolled f64 math (no_std: core has no sqrt/exp/ln/sin/cos; the
// floor/ceil/trunc/round families are re-implemented rather than relying on
// any std-only float method).
// ---------------------------------------------------------------------------
const LN2: f64 = 0.6931471805599453;
/// fdlibm split of ln2: hi has its low bits zeroed so `x - k*LN2_HI` is exact.
const LN2_HI: f64 = 6.93147180369123816490e-01;
const LN2_LO: f64 = 1.90821492927058770002e-10;
const PI: f64 = 3.141592653589793;
const PIO2_1: f64 = 1.57079632673412561417e+00;
const PIO2_1T: f64 = 6.07710050650619224932e-11;

pub fn f64_abs(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & 0x7fff_ffff_ffff_ffff)
}

/// Truncate toward zero by clearing the fractional mantissa bits.
pub fn f64_trunc(x: f64) -> f64 {
    if !(f64_abs(x) < 4.503599627370496e15) {
        return x; // >= 2^52 is already integral (also passes NaN/inf through)
    }
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    if e < 0 {
        return f64::from_bits(bits & 0x8000_0000_0000_0000);
    }
    f64::from_bits(bits & !(0x000f_ffff_ffff_ffffu64 >> e))
}

pub fn f64_floor(x: f64) -> f64 {
    let t = f64_trunc(x);
    if x < 0.0 && t != x {
        t - 1.0
    } else {
        t
    }
}

pub fn f64_ceil(x: f64) -> f64 {
    let t = f64_trunc(x);
    if x > 0.0 && t != x {
        t + 1.0
    } else {
        t
    }
}

/// Half-away-from-zero round (C `round`), used for range reduction only.
pub fn f64_round(x: f64) -> f64 {
    if x >= 0.0 {
        f64_floor(x + 0.5)
    } else {
        f64_ceil(x - 0.5)
    }
}

/// python `round(x, n)` equivalent for the decimal places the oracle reports.
pub fn round_to(x: f64, n: i32) -> f64 {
    let p = 10f64.powi_hint(n);
    f64_round(x * p) / p
}

/// Exact scaling by 2^k (k within +-1000 after the while-loops).
fn f64_scale2(x: f64, k: i32) -> f64 {
    let mut xx = x;
    let mut kk = k;
    while kk > 1000 {
        xx *= f64::from_bits((1023u64 + 1000) << 52);
        kk -= 1000;
    }
    while kk < -1000 {
        xx *= f64::from_bits((1023u64 - 1000) << 52);
        kk += 1000;
    }
    xx * f64::from_bits(((1023 + kk) as u64) << 52)
}

/// Newton sqrt with a mantissa-halving seed (sqrt(1)=1, sqrt(2)~1.5 seed).
pub fn f64_sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x.is_infinite() {
        return x;
    }
    if x < 2.2250738585072014e-308 {
        // subnormal: exact scale by 2^100 then back by 2^-50 (2^100 and 2^-50
        // are exactly representable, so the scaling is lossless)
        return f64_sqrt(x * 1.2676506002282294e30) * 8.881784197001252e-16;
    }
    let mut g = f64::from_bits((x.to_bits() >> 1) + (1023u64 << 51));
    for _ in 0..6 {
        g = 0.5 * (g + x / g);
    }
    g
}

/// exp(x) via k = round(x/ln2) reduction + Taylor on |r| <= ln2/2.
pub fn f64_exp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x > 709.782712893384 {
        return f64::INFINITY;
    }
    if x < -745.1332191019411 {
        return 0.0;
    }
    let k = f64_round(x / LN2);
    let r = (x - k * LN2_HI) - k * LN2_LO;
    // Horner for sum r^i / i!, i = 0..14
    let mut s = 1.0 / 87178291200.0;
    s = s * r + 1.0 / 6227020800.0;
    s = s * r + 1.0 / 479001600.0;
    s = s * r + 1.0 / 39916800.0;
    s = s * r + 1.0 / 3628800.0;
    s = s * r + 1.0 / 362880.0;
    s = s * r + 1.0 / 40320.0;
    s = s * r + 1.0 / 5040.0;
    s = s * r + 1.0 / 720.0;
    s = s * r + 1.0 / 120.0;
    s = s * r + 1.0 / 24.0;
    s = s * r + 1.0 / 6.0;
    s = s * r + 0.5;
    s = s * r + 1.0;
    s = s * r + 1.0;
    f64_scale2(s, k as i32)
}

/// ln(x) = e*ln2 + 2*atanh((m-1)/(m+1)) with m in [sqrt(1/2), sqrt(2)).
pub fn f64_ln(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x.is_infinite() {
        return x;
    }
    let bits = x.to_bits();
    if (bits >> 52) & 0x7ff == 0 {
        // subnormal: scale by 2^53 and correct the exponent
        return f64_ln(x * 9007199254740992.0) - 53.0 * LN2;
    }
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | (1023u64 << 52));
    if m > 1.4142135623730951 {
        m /= 2.0;
        e += 1;
    }
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    // sum_{j>=0} t2^j / (2j+1) for j = 6..0 (Horner)
    let mut s = 1.0 / 13.0;
    let mut i = 11;
    while i >= 1 {
        s = s * t2 + 1.0 / (i as f64);
        i -= 2;
    }
    2.0 * t * s + (e as f64) * LN2_HI + (e as f64) * LN2_LO
}

/// python math.log2(x) == ln(x)/ln(2).
pub fn f64_log2(x: f64) -> f64 {
    f64_ln(x) / LN2
}

/// x**n for the small integer powers the oracle uses (10**n, 2**k).
trait PowHint {
    fn powi_hint(self, n: i32) -> f64;
}
impl PowHint for f64 {
    fn powi_hint(self, n: i32) -> f64 {
        let mut r = 1.0f64;
        let mut k = if n < 0 { -n } else { n };
        let mut b = self;
        while k > 0 {
            if k & 1 == 1 {
                r *= b;
            }
            b *= b;
            k >>= 1;
        }
        if n < 0 {
            1.0 / r
        } else {
            r
        }
    }
}

fn sin_poly(r: f64) -> f64 {
    let r2 = r * r;
    let mut s = -7.647163731819816e-13;
    s = s * r2 + 1.6059043836821613e-10;
    s = s * r2 + -2.505210838544172e-8;
    s = s * r2 + 2.7557319223985893e-6;
    s = s * r2 + -1.984126984126984e-4;
    s = s * r2 + 0.008333333333333333;
    s = s * r2 + -0.16666666666666666;
    r + r * r2 * s
}

fn cos_poly(r: f64) -> f64 {
    let r2 = r * r;
    let mut s = 4.779477332387385e-14;
    s = s * r2 + -1.1470745597729725e-11;
    s = s * r2 + 2.08767569878681e-9;
    s = s * r2 + -2.755731922398589e-7;
    s = s * r2 + 2.48015873015873e-5;
    s = s * r2 + -0.001388888888888889;
    s = s * r2 + 0.041666666666666664;
    s = s * r2 + -0.5;
    1.0 + r2 * s
}

/// Cody-Waite quadrant reduction to |r| <= pi/4, then 4-way sign swap.
fn sin_cos_core(x: f64) -> (f64, f64) {
    let n = f64_round(x * (2.0 / PI));
    let r = (x - n * PIO2_1) - n * PIO2_1T;
    let (sr, cr) = (sin_poly(r), cos_poly(r));
    match ((n as i64) & 3) as i32 {
        0 => (sr, cr),
        1 => (cr, -sr),
        2 => (-sr, -cr),
        _ => (-cr, sr),
    }
}

pub fn f64_sin(x: f64) -> f64 {
    sin_cos_core(x).0
}

pub fn f64_cos(x: f64) -> f64 {
    sin_cos_core(x).1
}

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4).  Hand-rolled because python's hashlib is host-side and
// load_architecture needs `hashlib.sha256(pattern).hexdigest()[:12]`.
// ---------------------------------------------------------------------------
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg: Vec<u8> = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
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
    for i in 0..8 {
        out[4 * i..4 * i + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

/// python hashlib.sha256(b).hexdigest() (full 64 hex chars).
pub fn sha256_hexdigest(data: &[u8]) -> alloc::string::String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let d = sha256_digest(data);
    let mut s = alloc::string::String::with_capacity(64);
    for b in d.iter() {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

/// python hashlib.sha256(b).hexdigest()[:12] (the oracle's pattern_hash).
pub fn sha256_hex12(data: &[u8]) -> alloc::string::String {
    let mut s = sha256_hexdigest(data);
    s.truncate(12);
    s
}

// ─── the oracle's own 15 pattern factories ───
// Generated by parsing the oracle's own dict literals; each field is a literal, a parameter,
// a translated arithmetic/format expression, or a repeating byte period. Verified field-by-field
// against the live module (values, f-string renderings and expanded byte patterns).

#[derive(Clone, PartialEq, Debug)]
pub enum ArchField {
    I(i64),
    S(&'static str),
    W(String),
    L(&'static [&'static str]),
    Rep { period: &'static [u8], times: usize },
}

pub struct ArchDesc { pub fields: Vec<(&'static str, ArchField)> }

impl ArchDesc {
    pub fn field(&self, k: &str) -> Option<&ArchField> { self.fields.iter().find(|(n, _)| *n == k).map(|(_, v)| v) }
    pub fn text(&self, k: &str) -> Option<&str> { match self.field(k) { Some(ArchField::S(s)) => Some(s), Some(ArchField::W(s)) => Some(s.as_str()), _ => None } }
    pub fn int(&self, k: &str) -> Option<i64> { match self.field(k) { Some(ArchField::I(i)) => Some(*i), _ => None } }
    pub fn pattern_len(&self, k: &str) -> Option<usize> { match self.field(k) { Some(ArchField::Rep { period, times }) => Some(period.len() * times), _ => None } }
}

pub fn pattern_boltzmann_machine(units: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Boltzmann Stochastic Resonance")),
        ("paradigm", ArchField::S("Thermal Noise Stochastic Generative Modeling")),
        ("units", ArchField::I(units)),
        ("thermal_noise_source", ArchField::S("RowHammer / Transistor Thermal Drift")),
        ("distribution", ArchField::S("Gibbs-Boltzmann Equilibrium P(v,h)")),
        ("applications", ArchField::S("Generative Molecular Generation, Probabilistic Inference")),
        ("pattern", ArchField::Rep { period: &[0x33, 0xcc], times: 32 }),
    ]}
}

pub fn pattern_cellular_automaton(grid_size: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Cellular Automaton Grid")),
        ("paradigm", ArchField::S("Self-Healing & Defense Dynamics")),
        ("grid", ArchField::W(format!("{}×{}", grid_size, grid_size))),
        ("cells", ArchField::I(grid_size * grid_size)),
        ("rule", ArchField::S("Game of Life / Immune (B3/S23)")),
        ("neighbors_per_cell", ArchField::I(8)),
        ("pattern", ArchField::Rep { period: &[0x01], times: 64 }),
    ]}
}

pub fn pattern_content_addressable(entries: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Content-Addressable Memory (CAM)")),
        ("paradigm", ArchField::S("O(1) Parallel Key-Value Recall")),
        ("entries", ArchField::I(entries)),
        ("key_width", ArchField::I(32)),
        ("value_width", ArchField::I(32)),
        ("lookup_time", ArchField::S("O(1) — parallel comparison")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], times: 1 }),
    ]}
}

pub fn pattern_fpga(size_kb: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("FPGA-Lite")),
        ("paradigm", ArchField::S("Reconfigurable Logic & K3 Truth Tables")),
        ("logic_cells", ArchField::I((size_kb * 1024) / 64)),
        ("lut_size", ArchField::I(6)),
        ("total_gates", ArchField::I(65536)),
        ("pattern", ArchField::Rep { period: &[0xaa], times: 64 }),
    ]}
}

pub fn pattern_gnn_accelerator(nodes: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Graph Neural Network (GNN) Accelerator")),
        ("paradigm", ArchField::S("Relational Message Passing across Knowledge & Molecular Graphs")),
        ("nodes", ArchField::I(nodes)),
        ("edges_supported", ArchField::I(nodes * 16)),
        ("message_passing", ArchField::S("h_v^(k+1) = UPDATE(h_v^(k), AGGREGATE({h_u}))")),
        ("applications", ArchField::S("Totality 144.6M Records, Molecular Graphs, PPI Networks")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00], times: 1 }),
    ]}
}

pub fn pattern_holographic_associative(slots: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Holographic Associative Memory")),
        ("paradigm", ArchField::S("Distributed Optical-Analog Interference Memory Plates")),
        ("slots", ArchField::I(slots)),
        ("storage_density", ArchField::S("Complex Fourier Hologram Matrix")),
        ("fault_tolerance", ArchField::S("Graceful Degradation (0% Data Loss up to 40% Damage)")),
        ("applications", ArchField::S("Resilient Knowledge Shards, Holographic Recovery")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0xb4, 0x6b, 0x62, 0x74, 0x10, 0x12, 0x22, 0x9f, 0x43, 0x85, 0x3d, 0xdc, 0x17, 0x54, 0xa2, 0x7e, 0xbf, 0x34, 0x5f, 0xba, 0x02, 0x80, 0x53, 0xbb, 0xc7, 0x35, 0xcb, 0x7e, 0x3b, 0x53, 0x27, 0xdb, 0xf2, 0x84, 0xe1, 0x9f, 0x2f, 0x13, 0xda, 0x74, 0x16, 0x6b, 0xde, 0xfe, 0xb0, 0x93, 0x17, 0x8c, 0x10, 0xef, 0x9a, 0x61, 0x69, 0x7a, 0xac, 0x22, 0x0f, 0xab, 0x8a, 0x81, 0x49, 0xcc], times: 1 }),
    ]}
}

pub fn pattern_hopfield_attractor(neurons: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Hopfield Attractor Network")),
        ("paradigm", ArchField::S("Auto-Associative Basin Pattern Completion & Denoising")),
        ("neurons", ArchField::I(neurons)),
        ("energy_fn", ArchField::S("E = -1/2 Σ w_ij v_i v_j")),
        ("attractor_basins", ArchField::I(neurons / 8)),
        ("applications", ArchField::S("Corrupted Knowledge Repair, Associative Recognition")),
        ("pattern", ArchField::Rep { period: &[0x0f, 0xf0], times: 32 }),
    ]}
}

pub fn pattern_hyperdimensional_computing(dim: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Hyperdimensional Computing (HDC)")),
        ("paradigm", ArchField::S("High-Dimensional Vector Symbolic Architecture (VSA)")),
        ("hypervector_dim", ArchField::I(dim)),
        ("operations", ArchField::L(&["Binding (XOR)", "Bundling (Majority)", "Permutation (Shift)"])),
        ("distance_metric", ArchField::S("Hamming Distance Space")),
        ("applications", ArchField::S("Symbol Grounding, Cognitive Analogy, Zero-Shot Classification")),
        ("pattern", ArchField::Rep { period: &[0x5a, 0xa5], times: 32 }),
    ]}
}

pub fn pattern_ising_spin_glass(spins: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Ising Spin Glass")),
        ("paradigm", ArchField::S("Native Physical Spin Hamiltonian Optimization")),
        ("spins", ArchField::I(spins)),
        ("hamiltonian", ArchField::S("H = -Σ J_ij s_i s_j - Σ h_i s_i")),
        ("spin_precision", ArchField::S("Binary/Ternary Spin Lattice")),
        ("applications", ArchField::S("NP-Hard Optimization, Protein Pocket Folding")),
        ("pattern", ArchField::Rep { period: &[0x55, 0xaa], times: 32 }),
    ]}
}

pub fn pattern_neural_network(neurons: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("SRAM Neural Network")),
        ("paradigm", ArchField::S("Direct In-Cache Weight Inference")),
        ("neurons", ArchField::I(neurons)),
        ("weights_per_neuron", ArchField::I(16)),
        ("total_synapses", ArchField::I(neurons * 16)),
        ("precision", ArchField::S("16-bit fixed (per weight)")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x40], times: 32 }),
    ]}
}

pub fn pattern_oscillatory_neural_network(oscillators: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Oscillatory Neural Network (ONN)")),
        ("paradigm", ArchField::S("Coupled Kuramoto Phase Synchronization & Harmonic Resonance")),
        ("oscillators", ArchField::I(oscillators)),
        ("phase_equation", ArchField::S("dθ_i/dt = ω_i + Σ K_ij sin(θ_j - θ_i)")),
        ("frequencies_bound", ArchField::S("396 Hz - 963 Hz Sacred Geometric Solfeggio Lattices")),
        ("applications", ArchField::S("Spatiotemporal Pattern Binding, Direct Symbology Grounding")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0x80, 0x3f, 0x5e, 0x83, 0x6c, 0x3f, 0xf3, 0x04, 0x35, 0x3f, 0x15, 0xef, 0xc3, 0x3e, 0x32, 0x31, 0x8d, 0x24, 0x15, 0xef, 0xc3, 0xbe, 0xf3, 0x04, 0x35, 0xbf, 0x5e, 0x83, 0x6c, 0xbf, 0x00, 0x00, 0x80, 0xbf, 0x5e, 0x83, 0x6c, 0xbf, 0xf3, 0x04, 0x35, 0xbf, 0x15, 0xef, 0xc3, 0xbe, 0xca, 0xc9, 0x53, 0xa5, 0x15, 0xef, 0xc3, 0x3e, 0xf3, 0x04, 0x35, 0x3f, 0x5e, 0x83, 0x6c, 0x3f], times: 1 }),
    ]}
}

pub fn pattern_quantum_simulator(qubits: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Quantum Circuit Simulator")),
        ("paradigm", ArchField::S("Multi-Hypothesis Superposition & Entanglement")),
        ("qubits", ArchField::I(qubits)),
        ("state_vector_size", ArchField::I(2048)),
        ("gates", ArchField::L(&["H", "CNOT", "T", "X", "Z", "S"])),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], times: 2048 }),
    ]}
}

pub fn pattern_sparse_attention(context_length: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Sparse Transformer Attention")),
        ("paradigm", ArchField::S("Selective Long-Context Block-Sparse Attention Routing")),
        ("context_length", ArchField::I(context_length)),
        ("attention_sparsity", ArchField::S("93.75% Sparse Block Routing")),
        ("attention_formula", ArchField::S("Attention = softmax(QK^T / √d) V")),
        ("applications", ArchField::S("7M ARC Geometric Model, Long-Span Narrative Reasoning")),
        ("pattern", ArchField::Rep { period: &[0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01], times: 8 }),
    ]}
}

pub fn pattern_systolic_array(pe_grid: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Systolic Matrix Array")),
        ("paradigm", ArchField::S("Rhythmic 2D Dataflow Matrix Multiply Acceleration")),
        ("pe_dimensions", ArchField::W(format!("{}x{} PEs", pe_grid, pe_grid))),
        ("total_pes", ArchField::I(pe_grid * pe_grid)),
        ("dataflow", ArchField::S("Output-Stationary Wavefront")),
        ("applications", ArchField::S("Sub-microsecond AutoDock Vina Matrix Multiplication")),
        ("pattern", ArchField::Rep { period: &[0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x3f, 0xab, 0xaa, 0xaa, 0x3e, 0x00, 0x00, 0x80, 0x3e, 0xcd, 0xcc, 0x4c, 0x3e, 0xab, 0xaa, 0x2a, 0x3e, 0x25, 0x49, 0x12, 0x3e, 0x00, 0x00, 0x00, 0x3e, 0x39, 0x8e, 0xe3, 0x3d, 0xcd, 0xcc, 0xcc, 0x3d, 0x8c, 0x2e, 0xba, 0x3d, 0xab, 0xaa, 0xaa, 0x3d, 0xd9, 0x89, 0x9d, 0x3d, 0x25, 0x49, 0x92, 0x3d, 0x89, 0x88, 0x88, 0x3d, 0x00, 0x00, 0x80, 0x3d], times: 1 }),
    ]}
}

pub fn pattern_tcam(entries: i64) -> ArchDesc {
    ArchDesc { fields: vec![
        ("name", ArchField::S("Ternary Content-Addressable Memory (TCAM)")),
        ("paradigm", ArchField::S("K3-Native Three-State Wildcard Matching (+1, 0, -1)")),
        ("entries", ArchField::I(entries)),
        ("bits_per_trit", ArchField::I(2)),
        ("match_logic", ArchField::S("Match = ∏ (x_i == y_i OR x_i == UNKNOWN)")),
        ("applications", ArchField::S("K3 Logic Rules, IP Routing, Fuzzy Chemical Substructures")),
        ("pattern", ArchField::Rep { period: &[0x81, 0x42, 0x24, 0x18], times: 16 }),
    ]}
}

pub fn py_pow_i64(base: i64, exp: i64) -> i64 { let mut r = 1i64; let mut i = 0; while i < exp { r *= base; i += 1; } r }

// ═══════════════════════════════════════════════════════════════════════════════
// Hopfield RELAXATION — the recall dynamics, as distinct from the one-step update above.
//
// `sram_lut_hopfield_recall` applies ONE synchronous update: s' = sign(W s). That is a single step of the
// dynamics, not associative recall -- a Hopfield network recalls by iterating until the state stops moving,
// and the energy along the way is the evidence that it is settling rather than cycling. What was missing in
// this tree was that iteration, so "recall" could not recover anything.
//
// The update here is ASYNCHRONOUS (Gauss-Seidel: one unit at a time, in index order, using already-updated
// values). With a symmetric, zero-diagonal W that cannot increase E(s) = -Σ_ij w_ij s_i s_j, so the energy
// trace is monotone non-increasing by construction -- and the ruler asserts it rather than trusting this
// comment. E is the usual energy up to the 1/2 factor, kept as exact integer arithmetic so the Python mirror
// in swarm_v2/core/sram_pattern_computer.py agrees bit-for-bit instead of roughly.
// ═══════════════════════════════════════════════════════════════════════════════

/// Store `patterns` (each `dim` long, values ±1) as Hebbian weights, flattened row-major dim×dim.
/// w_ij = Σ_p ξ_i^p ξ_j^p, zero diagonal, scaled by `GAIN` and clamped to i8. The gain and the clamp are
/// explicit constants so the Python mirror cannot silently disagree.
pub const HOPFIELD_GAIN: i32 = 8;

pub fn hopfield_weights(patterns: &[Vec<i8>], dim: usize) -> Vec<i8> {
    let mut w = vec![0i8; dim * dim];
    for p in patterns {
        for i in 0..dim {
            for j in 0..dim {
                if i != j {
                    let prod = (p[i] as i32) * (p[j] as i32);
                    let v = w[i * dim + j] as i32 + prod;
                    w[i * dim + j] = v.clamp(-127, 127) as i8;
                }
            }
        }
    }
    for i in 0..dim {
        w[i * dim + i] = 0;
    }
    // the gain is applied after the sum, so accumulation cannot saturate differently per pattern
    for v in w.iter_mut() {
        *v = ((*v as i32) * HOPFIELD_GAIN).clamp(-127, 127) as i8;
    }
    w
}

/// E(s) = -Σ_ij w_ij s_i s_j, exact integers (the usual energy up to the 1/2 factor).
pub fn hopfield_energy(state: &[i8], weights: &[i8], dim: usize) -> i64 {
    let mut e: i64 = 0;
    for i in 0..dim {
        for j in 0..dim {
            e -= (weights[i * dim + j] as i64) * (state[i] as i64) * (state[j] as i64);
        }
    }
    e
}

/// The result of a relaxation: the settled state and the evidence that it settled.
pub struct Relaxation {
    pub state: Vec<i8>,
    pub sweeps: u64,
    pub converged: bool,
    pub initial_energy: i64,
    pub final_energy: i64,
    pub energy_trace: Vec<i64>,
}

/// Relax `state` to a fixed point of the weights: one unit at a time, in index order, until a whole sweep
/// changes nothing (`converged`) or `max_sweeps` is reached. The trace holds the energy after each sweep.
pub fn hopfield_relax(state: &[i8], weights: &[i8], dim: usize, max_sweeps: usize) -> Relaxation {
    let mut s = vec![0i8; dim];
    for i in 0..dim {
        s[i] = if state[i % state.len()] >= 0 { 1 } else { -1 };
    }
    let initial_energy = hopfield_energy(&s, weights, dim);
    let mut trace = vec![initial_energy];
    let mut converged = false;
    let mut sweeps = 0u64;

    while sweeps < max_sweeps as u64 {
        let mut changed = 0u64;
        for i in 0..dim {
            let mut field: i64 = 0;
            for j in 0..dim {
                if j != i {
                    field += (weights[i * dim + j] as i64) * (s[j] as i64);
                }
            }
            let want: i8 = if field >= 0 { 1 } else { -1 };
            if want != s[i] {
                s[i] = want;
                changed += 1;
            }
        }
        sweeps += 1;
        trace.push(hopfield_energy(&s, weights, dim));
        if changed == 0 {
            converged = true;
            break;
        }
    }

    Relaxation {
        final_energy: *trace.last().unwrap_or(&initial_energy),
        state: s,
        sweeps,
        converged,
        initial_energy,
        energy_trace: trace,
    }
}

/// Fraction of positions where two states differ -- the recall metric the tests report in.
pub fn hopfield_hamming(a: &[i8], b: &[i8]) -> u64 {
    a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as u64
}

// ── added here from this machine's tree (G194) ────────────────────────────────────────────────────
/// Ternary CAM match: every non-wildcard position must equal the stored trit.
/// Added from the newer tree - `k3_ternary_sram` imports this name and this file did not have it.
pub fn pattern_tcam_match(stored_trits: &[i8], query_trits: &[i8], mask: &[u8]) -> bool {
    let n = stored_trits.len().min(query_trits.len());
    for i in 0..n {
        let is_wildcard = mask.get(i).copied().unwrap_or(0) == 0;
        if !is_wildcard && stored_trits[i] != query_trits[i] {
            return false;
        }
    }
    true
}

// ── behaviour the newer tree has under names this file did not use (G194) ─────────────────────────
/// Count rows equal to `key` (content-addressable match).
/// `pattern_content_addressable` in this file takes an `i64` and returns an `ArchDesc` - an architecture
/// spec, not a matcher - so the matching behaviour arrives under its own name instead of colliding with it.
pub fn cam_match_count(rows: &[&[u8]], key: &[u8]) -> u64 {
    rows.iter()
        .filter(|r| r.len() == key.len() && r.iter().zip(key.iter()).all(|(a, b)| a == b))
        .count() as u64
}

/// One Rule-30 step over a byte ring, returning the evolved bytes.
/// Likewise named apart from this file's `pattern_cellular_automaton(i64) -> ArchDesc`.
pub fn cellular_evolve_bytes(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let mut out = vec![0u8; n];
    for i in 0..n {
        let left = input[(i + n - 1) % n];
        let center = input[i];
        let right = input[(i + 1) % n];
        let a = (left >> 7) & 1;
        let b = (center >> 7) & 1;
        let c = (right >> 7) & 1;
        let rule30 = (a ^ (b | c)) as u8;
        out[i] = (center << 1) | rule30;
    }
    out
}
