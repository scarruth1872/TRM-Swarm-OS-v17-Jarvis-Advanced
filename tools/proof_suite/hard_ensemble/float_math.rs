//! Dependency-free float math for the no_std kernel.
//!
//! The kernel has no libm, so f64::exp / .round / .ln do not exist.
//! These are the primitives the instruction set needed to be ported.

/// Round to nearest integer, ties away from zero (f64::round semantics).
pub fn round_f64(x: f64) -> f64 {
    if x >= 0.0 { (x + 0.5) as i64 as f64 } else { -((-x + 0.5) as i64 as f64) }
}

/// 2^k for integer k, via IEEE-754 exponent construction (no libm).
pub fn pow2i(k: i32) -> f64 {
    if k > 1023 { return f64::INFINITY; }
    if k < -1074 { return 0.0; }
    if k >= -1022 {
        f64::from_bits((((k + 1023) as u64) & 0x7FF) << 52)
    } else {
        // subnormal range: split the shift so it stays exact
        f64::from_bits(1u64 << (k + 1074)) * pow2i(-1022)
    }
}

const LN2: f64 = 0.6931471805599453;
const INV_LN2: f64 = 1.4426950408889634;

/// e^x. Range-reduced: x = k*ln2 + f, then a series on f in [-ln2/2, ln2/2].
///
/// ACCURACY (measured, not asserted): the first version stopped at f^7/7! and
/// achieved only ~5e-9 relative - the truncated term is f^8/8! = 0.347^8/40320
/// = 5.2e-9 at the range boundary, which matched the measurement. Terms through
/// f^11/11! bring it to ~6e-15 (next term f^12/12! = 6.4e-15). Verified against
/// known values; see the porting test.
pub fn exp_f64(x: f64) -> f64 {
    if x != x { return x; }                  // NaN
    if x > 709.782712893384 { return f64::INFINITY; }
    if x < -745.1332191019411 { return 0.0; }
    let k = round_f64(x * INV_LN2);
    let f = x - k * LN2;
    let p = 1.0
        + f * (1.0
        + f * (0.5
        + f * (1.0 / 6.0
        + f * (1.0 / 24.0
        + f * (1.0 / 120.0
        + f * (1.0 / 720.0
        + f * (1.0 / 5040.0
        + f * (1.0 / 40320.0
        + f * (1.0 / 362880.0
        + f * (1.0 / 3628800.0
        + f * (1.0 / 39916800.0)))))))))));
    p * pow2i(k as i32)
}

/// e^x as f32.
pub fn exp_f32(x: f32) -> f32 { exp_f64(x as f64) as f32 }

// ── Deterministic PRNG ───────────────────────────────────────────────────────
// The Python originals used np.random.randn, which is NON-deterministic - two
// runs of the same instruction produced different numbers, so its benchmark
// could never be reproduced. xorshift64* is used instead: same weights on every
// boot, on every machine, which makes the port reproducible where Python was not.

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // avoid the all-zero fixed point
        Rng(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    /// Uniform in [0,1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }
    /// Approximate N(0,1) via the sum of 12 uniforms minus 6 (Irwin-Hall).
    /// Bounded to ~±6 sigma, which is ample for weight initialisation.
    pub fn next_normal(&mut self) -> f64 {
        let mut s = 0.0;
        for _ in 0..12 { s += self.next_f64(); }
        s - 6.0
    }
}

// ── Softmax / top-k over expert scores ───────────────────────────────────────

/// Numerically-stable softmax. Subtracts the max before exponentiating, so it
/// cannot overflow even for large scores.
pub fn softmax(scores: &[f64], out: &mut [f64]) {
    let n = scores.len().min(out.len());
    if n == 0 { return; }
    let mut mx = f64::NEG_INFINITY;
    for i in 0..n { if scores[i] > mx { mx = scores[i]; } }
    let mut sum = 0.0;
    for i in 0..n { out[i] = exp_f64(scores[i] - mx); sum += out[i]; }
    if sum == 0.0 { return; }
    for i in 0..n { out[i] /= sum; }
}

/// Indices of the two largest values, descending, plus their sum.
/// Ties resolve to the lower index, so the result is deterministic.
pub fn top2_indices(scores: &[f64]) -> (u64, u64, f64) {
    let mut i0: usize = usize::MAX;
    let mut i1: usize = usize::MAX;
    let mut v0 = f64::NEG_INFINITY;
    let mut v1 = f64::NEG_INFINITY;
    for (i, &v) in scores.iter().enumerate() {
        if v > v0 {
            v1 = v0; i1 = i0;
            v0 = v;  i0 = i;
        } else if v > v1 {
            v1 = v;  i1 = i;
        }
    }
    if i0 == usize::MAX { return (0, 0, 0.0); }
    let second = if i1 == usize::MAX { 0 } else { i1 as u64 };
    (i0 as u64, second, v0 + v1)
}

/// Dot product of two equal-length slices (shorter length wins).
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let mut s = 0.0;
    for i in 0..n { s += a[i] * b[i]; }
    s
}

/// Square root. no_std has no `f64::sqrt`, so: bit-trick seed then Newton
/// iterations. Newton doubles the correct digits each step, so four iterations
/// from a ~4-bit seed reach full f64 precision - measured, not assumed.
///
/// Moved here from a private `SqrtHack` trait in star_matrix_lattice.rs so
/// every module shares one verified implementation.
pub fn sqrt_f64(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    // scale into [1,4) so the exponent trick is well conditioned
    let mut scale = 1.0f64;
    let mut v = x;
    while v >= 4.0 { v *= 0.25; scale *= 2.0; }
    while v < 1.0 { v *= 4.0; scale *= 0.5; }
    // initial guess ~ sqrt(v) for v in [1,4)
    let mut g = 0.5 * (1.0 + v / 2.0);
    for _ in 0..6 {
        g = 0.5 * (g + v / g);
    }
    g * scale
}
