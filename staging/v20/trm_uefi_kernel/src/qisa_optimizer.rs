//! TRM Swarm OS - qisa_optimizer (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/qisa_optimizer.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  AnnealingConfig 
#[derive(Debug, Clone)]
pub struct AnnealingConfig {
    pub initial_temperature: f64,
    pub cooling_rate: f64,
    pub min_temperature: f64,
    pub max_cycles: i64,
    pub tunnel_probability: f64,
    pub tunnel_strength: f64,
    pub convergence_threshold: f64,
}

impl AnnealingConfig {
    /// The oracle dataclass defaults (`AnnealingConfig`, qisa_optimizer.py:60-66).
    /// These were all zero in the mechanical extraction, which silently kills every
    /// annealing run (0 cycles, no probability mass) instead of running 50.
    pub fn new() -> Self {
        AnnealingConfig {
            initial_temperature: 1.0,
            cooling_rate: 0.95,
            min_temperature: 0.01,
            max_cycles: 50,
            tunnel_probability: 0.15,
            tunnel_strength: 0.3,
            convergence_threshold: 0.001,
        }
    }
}

//  AnnealingResult 
#[derive(Debug, Clone)]
pub struct AnnealingResult {
    pub best_index: i64,
    pub best_score: f64,
    pub initial_best_score: f64,
    pub cycles_run: i64,
    pub tunneling_events: i64,
    pub temperature_final: f64,
    pub history: Vec<Vec<(String, f64)>>,
}

impl AnnealingResult {
    pub fn new() -> Self {
        AnnealingResult {
            best_index: 0,
            best_score: 0.0,
            initial_best_score: 0.0,
            cycles_run: 0,
            tunneling_events: 0,
            temperature_final: 0.0,
            history: vec![],
        }
    }

    /// `AnnealingResult.improvement_pct()` (qisa_optimizer.py:80-84): the division is
    /// guarded so a zero starting score yields 0.0 instead of inf/NaN, and the
    /// denominator is `abs(initial_best_score)`, not the signed value.
    pub fn improvement_pct(&self) -> f64 {
        if self.initial_best_score == 0.0 {
            return 0.0;
        }
        ((self.best_score - self.initial_best_score) / self.initial_best_score.abs()) * 100.0
    }
}

//  get_qisa_optimizer(config) 
static SINGLETON_QISA_OPTIMIZER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_qisa_optimizer() -> u64 {
    // Singleton handle (lazy-init pattern from qisa_optimizer.py).
    // Returns 1 once configured; 0 until first call initializes.
    // NOTE: the oracle's get_qisa_optimizer() (qisa_optimizer.py:270-274) returns the
    // memoized QISAOptimizer *instance*; a u64 handle cannot carry one and no rule of
    // the oracle is computed here.  Construct `QisaOptimizer::new(..)` below instead.
    let v = SINGLETON_QISA_OPTIMIZER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_QISA_OPTIMIZER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// =============================================================================
// Port of QISAOptimizer.optimize / .get_stats / AnnealingResult.improvement_pct
// Oracle: swarm_v2/core/qisa_optimizer.py -- AnnealingConfig 60-66, AnnealingResult
// 69-84, QISAOptimizer.optimize 105-212, get_stats 250-264.
//
// Entropy is HOST-SIDE in the oracle: it draws from the process-global CPython
// `random` module (random.random / random.randint / random.gauss).  The port takes
// that source as an explicit `QisaRng` parameter (`SplitMix64Rng` is the kernel
// default) and reproduces exactly what the oracle does *with* the draws.  The
// oracle's logger.debug/info calls and optimize_qstate (torch + ManusProtocol
// tensor plumbing) are host-side and are not ported.
// =============================================================================

/// CPython `max(a, b)`: `b` only when `b > a`, so a NaN first argument propagates
/// (`max(0.0, NaN) == 0.0`).  Same helper as architectural_rl_agent.rs::py_max.
#[inline]
fn py_max(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// CPython `round(x, 4)` (half-to-even on the exact binary value, via the decimal
/// round trip -- the technique architectural_rl_agent.rs::round_3 uses).  The oracle
/// rounds temperature/best_score *before storing them* (qisa_optimizer.py:176-178) and
/// then runs its convergence test on those rounded values (187-189), so this must be
/// `round`, not a truncation.
pub fn round_4(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    match format!("{:.4}", x).parse::<f64>() {
        Ok(v) => v,
        Err(_) => x,
    }
}

/// 2^e for e in [-1022, 1023] by bit construction (no std::ldexp).
#[inline]
fn pow2(e: i32) -> f64 {
    f64::from_bits((((e + 1023) as i64) as u64) << 52)
}

/// m * 2^e without std::ldexp.  When 2^e is below the normal range the scale is folded
/// in while the product is still normal and then by 2^-1022, so a subnormal result is
/// rounded once instead of twice.
fn ldexp_f64(m: f64, mut e: i32) -> f64 {
    let mut r = m;
    while e > 1023 {
        r *= pow2(1023);
        e -= 1023;
    }
    while e < -1022 {
        let step = if e + 1022 >= -1022 { e + 1022 } else { -1022 };
        r *= pow2(step);
        e -= step;
    }
    r * pow2(e)
}

/// `math.exp` for f64 without libm.  Same decomposition as the kernel's
/// mamba3_turbovec_ssm::f64_exp: exp(x) = 2^k * exp(r) with r = x - k*ln2 computed
/// against a two-part ln2, |r| <= 0.3466, Taylor series to r^13.
pub fn exp_f64(x: f64) -> f64 {
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
    // round-to-nearest integer without std::round (2^52 add/subtract trick)
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
    ldexp_f64(p, k as i32)
}

/// sqrt(f64) by bit-halving seed + Newton iterations (same routine as
/// mamba3_turbovec_ssm::f64_sqrt: ~1e-2 -> ~1e-16 in 6 steps).
pub fn f64_sqrt(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x != x || x == f64::INFINITY {
        return x;
    }
    // The bit-halving seed assumes a normal exponent field, so subnormal input is
    // scaled through 2^64 first and the root back by 2^-32 (both exact powers of two).
    let (v, scale) = if x < f64::MIN_POSITIVE {
        (x * 18446744073709551616.0, 1.0 / 4294967296.0)
    } else {
        (x, 1.0)
    };
    let mut y = f64::from_bits((v.to_bits() >> 1) + 0x1ff8_0000_0000_0000);
    let mut i = 0;
    while i < 6 {
        y = 0.5 * (y + v / y);
        i += 1;
    }
    y * scale
}

/// ln(f64) without libm: x = m * 2^e with m in [1/sqrt2, sqrt2), and
/// ln(x) = e*ln2 + 2*atanh(t), t = (m-1)/(m+1) in [-0.1716, 0.1716], where the atanh
/// series is summed to t^19/19 (truncation < 4e-18).
pub fn f64_ln(x: f64) -> f64 {
    if x != x {
        return x;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x < 0.0 {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return x;
    }
    const LN2: f64 = 6.931_471_805_599_453e-1;
    const SQRT2: f64 = 1.414_213_562_373_095_1;
    let bits = x.to_bits();
    let mut e = (((bits >> 52) & 0x7ff) as i32) - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | (1023u64 << 52));
    if (bits >> 52) & 0x7ff == 0 {
        // subnormal: no implicit leading bit, so normalise through 2^64 first
        let b2 = (x * 18446744073709551616.0).to_bits();
        e = (((b2 >> 52) & 0x7ff) as i32) - 1023 - 64;
        m = f64::from_bits((b2 & 0x000f_ffff_ffff_ffff) | (1023u64 << 52));
    }
    if m >= SQRT2 {
        m *= 0.5;
        e += 1;
    }
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let mut term = 1.0;
    let mut s = 1.0;
    let mut k = 3.0;
    let mut i = 0;
    while i < 9 {
        term *= t2;
        s += term / k;
        k += 2.0;
        i += 1;
    }
    2.0 * t * s + (e as f64) * LN2
}

/// The oracle's entropy source, injected.  `next_unit` is `random.random()` ([0,1)),
/// `next_index(n)` is `random.randint(0, n-1)`, `next_gauss(sigma)` is
/// `random.gauss(0, sigma)`.
pub trait QisaRng {
    fn next_unit(&mut self) -> f64;
    fn next_index(&mut self, n: usize) -> usize;
    fn next_gauss(&mut self, sigma: f64) -> f64;
}

/// Kernel default entropy source: SplitMix64 with CPython's `_randbelow` rejection
/// shape.  The oracle's stream comes from CPython's MT19937 (process-global state), so
/// the *sequences* differ by construction -- only the policy around the draws is the
/// oracle's.  `next_gauss` uses the Marsaglia polar pair (ln + sqrt, no trig) instead
/// of CPython's sin/cos Box-Muller: both produce an exact standard normal, and the
/// paired second value is cached exactly as `random.gauss` caches `gauss_next`.
pub struct SplitMix64Rng {
    state: u64,
    gauss_next: Option<f64>,
}

impl SplitMix64Rng {
    pub fn new(seed: u64) -> Self {
        SplitMix64Rng {
            state: seed,
            gauss_next: None,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// CPython `getrandbits(k)`: k bits from the top of one draw.
    fn getrandbits(&mut self, k: u32) -> u64 {
        if k == 0 {
            0
        } else {
            self.next_u64() >> (64 - k)
        }
    }
}

impl QisaRng for SplitMix64Rng {
    fn next_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }

    fn next_index(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0; // callers guarantee n >= 1 (empty scores are rejected first)
        }
        let k = 64 - (n as u64).leading_zeros(); // n.bit_length()
        loop {
            let r = self.getrandbits(k);
            if r < n as u64 {
                return r as usize;
            }
        }
    }

    fn next_gauss(&mut self, sigma: f64) -> f64 {
        if let Some(z) = self.gauss_next.take() {
            return z * sigma;
        }
        loop {
            let u = 2.0 * self.next_unit() - 1.0;
            let v = 2.0 * self.next_unit() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let m = f64_sqrt(-2.0 * f64_ln(s) / s);
                self.gauss_next = Some(v * m);
                return u * m * sigma;
            }
        }
    }
}

/// `optimize()` on an empty score list raises `ValueError` in the oracle
/// (qisa_optimizer.py:118-119); the port returns an error instead of panicking.
#[derive(Debug, Clone, PartialEq)]
pub enum QisaError {
    EmptyScoreList,
}

/// `history[-k][key]` -- the oracle keeps history entries as dicts
/// (qisa_optimizer.py:174-181) and reads them back by key at 179-180 and 187-189.
/// The port keeps the `Vec<(String, f64)>` entry shape declared above; the integer
/// fields (cycle, best_idx) and the `tunneled` flag are exact in f64.
#[inline]
pub fn hist_get(entry: &[(String, f64)], key: &str) -> f64 {
    for (k, v) in entry {
        if k.as_str() == key {
            return *v;
        }
    }
    0.0
}

/// `get_stats()` (qisa_optimizer.py:250-264); the oracle returns a dict, the port a
/// struct with the same keys (its `config` sub-dict is flattened).
#[derive(Debug, Clone)]
pub struct QisaStats {
    pub total_runs: i64,
    pub total_tunneling_events: i64,
    pub avg_improvement_pct: f64,
    pub initial_temperature: f64,
    pub cooling_rate: f64,
    pub tunnel_probability: f64,
    pub max_cycles: i64,
}

/// Port of `QISAOptimizer` (qisa_optimizer.py:87-264).  `run_count`/`total_tunnels`
/// are the oracle's `_run_count`/`_total_tunnels`, `history` its `_history`.
#[derive(Debug, Clone)]
pub struct QisaOptimizer {
    pub config: AnnealingConfig,
    pub run_count: i64,
    pub total_tunnels: i64,
    pub history: Vec<AnnealingResult>,
}

impl QisaOptimizer {
    /// `QISAOptimizer.__init__` (qisa_optimizer.py:99-103).
    pub fn new(config: AnnealingConfig) -> Self {
        QisaOptimizer {
            config,
            run_count: 0,
            total_tunnels: 0,
            history: Vec::new(),
        }
    }

    /// `QISAOptimizer.optimize(scores, score_fn)` (qisa_optimizer.py:105-212).  `rng`
    /// carries the oracle's `random` draws; pass `None` for `score_fn` to use the
    /// plain `current_scores[idx] + perturbation` branch (line 147).
    pub fn optimize(
        &mut self,
        scores: &[f64],
        score_fn: Option<&dyn Fn(usize) -> f64>,
        rng: &mut dyn QisaRng,
    ) -> Result<AnnealingResult, QisaError> {
        if scores.is_empty() {
            return Err(QisaError::EmptyScoreList);
        }

        let cfg = self.config.clone();
        let n = scores.len();
        let mut current_scores: Vec<f64> = scores.to_vec();
        // max(range(n), key=...) -- CPython's max keeps the FIRST maximal index.
        let mut current_best_idx = 0usize;
        let mut i = 1usize;
        while i < n {
            if current_scores[i] > current_scores[current_best_idx] {
                current_best_idx = i;
            }
            i += 1;
        }
        let mut current_best_score = current_scores[current_best_idx];
        let initial_best_score = current_best_score;

        let mut temperature = cfg.initial_temperature;
        let mut tunneling_events: i64 = 0;
        let mut history: Vec<Vec<(String, f64)>> = Vec::new();

        // for cycle in range(cfg.max_cycles): a non-positive max_cycles runs zero cycles.
        let cycles: u64 = if cfg.max_cycles > 0 {
            cfg.max_cycles as u64
        } else {
            0
        };
        let mut cycle: u64 = 0;
        while cycle < cycles {
            if temperature < cfg.min_temperature {
                break;
            }

            // --- Quantum Tunneling (lines 136-152) ---
            if rng.next_unit() < cfg.tunnel_probability {
                let tunnel_idx = rng.next_index(n);
                let perturbation = rng.next_gauss(cfg.tunnel_strength);
                let new_score = match score_fn {
                    Some(f) => f(tunnel_idx) + perturbation,
                    None => current_scores[tunnel_idx] + perturbation,
                };
                current_scores[tunnel_idx] = py_max(0.0, new_score);
                tunneling_events += 1;
            }

            // --- Simulated Annealing Step (lines 154-171) ---
            let candidate_idx = rng.next_index(n);
            let candidate_score = current_scores[candidate_idx];
            let delta = candidate_score - current_best_score;

            if delta > 0.0 {
                current_best_idx = candidate_idx;
                current_best_score = candidate_score;
            } else {
                let acceptance_prob = exp_f64(delta / py_max(temperature, 1e-10));
                if rng.next_unit() < acceptance_prob {
                    current_best_idx = candidate_idx;
                    current_best_score = candidate_score;
                }
            }

            // --- Record cycle (lines 173-181) ---
            // The oracle's `tunneled` expression is `tunneling_events > (len(history)
            // and history[-1].get("tunneled", 0) if history else 0)`: with an empty
            // history the right side is the int 0, otherwise the previous entry's bool.
            let prev_tunneled = match history.last() {
                Some(h) => hist_get(h, "tunneled"),
                None => 0.0,
            };
            history.push(vec![
                (String::from("cycle"), cycle as f64),
                (String::from("temperature"), round_4(temperature)),
                (String::from("best_idx"), current_best_idx as f64),
                (String::from("best_score"), round_4(current_best_score)),
                (
                    String::from("tunneled"),
                    if (tunneling_events as f64) > prev_tunneled {
                        1.0
                    } else {
                        0.0
                    },
                ),
            ]);

            // --- Cooling (line 184) ---
            temperature *= cfg.cooling_rate;

            // --- Convergence check (lines 186-191) ---
            if history.len() >= 5 {
                let last = hist_get(&history[history.len() - 1], "best_score");
                let back5 = hist_get(&history[history.len() - 5], "best_score");
                if (last - back5).abs() < cfg.convergence_threshold {
                    break;
                }
            }

            cycle += 1;
        }

        let result = AnnealingResult {
            best_index: current_best_idx as i64,
            best_score: current_best_score,
            initial_best_score,
            cycles_run: history.len() as i64,
            tunneling_events,
            temperature_final: temperature,
            history,
        };

        self.run_count += 1;
        self.total_tunnels += tunneling_events;
        self.history.push(result.clone());

        Ok(result)
    }

    /// `get_stats()` (qisa_optimizer.py:250-264): the average is over `max(1, len)`,
    /// so an empty history averages to 0.0 rather than dividing by zero.
    pub fn get_stats(&self) -> QisaStats {
        let mut sum = 0.0;
        for r in self.history.iter() {
            sum += r.improvement_pct();
        }
        QisaStats {
            total_runs: self.run_count,
            total_tunneling_events: self.total_tunnels,
            avg_improvement_pct: sum / py_max(self.history.len() as f64, 1.0),
            initial_temperature: self.config.initial_temperature,
            cooling_rate: self.config.cooling_rate,
            tunnel_probability: self.config.tunnel_probability,
            max_cycles: self.config.max_cycles,
        }
    }
}
