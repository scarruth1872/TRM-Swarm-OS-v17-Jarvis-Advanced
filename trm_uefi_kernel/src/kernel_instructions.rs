//! TRM Swarm OS — Custom Kernel Instructions, software-emulation layer (no_std port).
//! 1:1 core semantics from swarm_v2/core/kernel_instructions.py.
//! The 8 proposed silicon instructions: the deterministic scalar/string kernels
//! (K3CLASS, PONDEROMOTIVE, TURBOVEC_DOT, DDR_MATCH, RADIX_TRIE_WALK.optimized)
//! are ported faithfully with pure f64/i64/string arithmetic — no libm, no
//! numpy. The numpy-matrix classes (MAMBA3STEP, LORA_FUSED, INSTELLA_GATE) are
//! initialized from np.random.randn and benchmark_instruction() is a
//! perf_counter timing loop: deep-Python bodies → honest // MANUAL stubs.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// no_std floor (truncation + sign correction, same helper as thz_wave_gateway.rs).
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < t {
        t - 1.0
    } else {
        t
    }
}

/// round-to-1-decimal for non-negative values (python round(v, 1); half-up —
/// identical for all values not exactly on a .05 tie).
fn round1(v: f64) -> f64 {
    floor_f64(v * 10.0 + 0.5) / 10.0
}

/// ---------------------------------------------------------------------------
/// Instruction 1: K3CLASS — ternary K3 classification {-1, 0, +1} from a
/// real-valued vector (1:1 python; both numpy paths produce identical output).
/// ---------------------------------------------------------------------------
pub struct K3CLASS;

impl K3CLASS {
    /// current(): result = 0; 1 where x > threshold; -1 where x < -threshold.
    pub fn current(x: &[f64], threshold: f64) -> Vec<i8> {
        x.iter()
            .map(|v| {
                if *v > threshold {
                    1
                } else if *v < -threshold {
                    -1
                } else {
                    0
                }
            })
            .collect()
    }

    /// optimized(): np.sign(x) * (|x| > threshold) — identical output to
    /// current() for all finite inputs (1:1 python).
    pub fn optimized(x: &[f64], threshold: f64) -> Vec<i8> {
        x.iter()
            .map(|v| {
                if *v > threshold {
                    1
                } else if *v < -threshold {
                    -1
                } else {
                    0
                }
            })
            .collect()
    }
}

/// ---------------------------------------------------------------------------
/// Instruction 3: PONDEROMOTIVE — classify wave envelope into {-1,0,+1}
/// buckets. current() uses strict thresholds; optimized() replicates the numpy
/// np.digitize(bins=[-inf,-0.5,0.5,inf]) bucket semantics exactly, where the
/// half-open interval puts x == +0.5 into the +1 bucket (1:1 python).
/// ---------------------------------------------------------------------------
pub struct PONDEROMOTIVE;

impl PONDEROMOTIVE {
    /// current(): e > 0.5 -> 1; e < -0.5 -> -1; else 0.
    pub fn current(envelope: &[f64]) -> Vec<i8> {
        envelope
            .iter()
            .map(|e| {
                if *e > 0.5 {
                    1
                } else if *e < -0.5 {
                    -1
                } else {
                    0
                }
            })
            .collect()
    }

    /// optimized(): numpy digitize semantics — e >= 0.5 -> 1; e < -0.5 -> -1;
    /// else 0. (np.digitize returns bin index i where bins[i-1] <= x < bins[i];
    /// labels[indices-1] therefore maps +0.5 to +1 and -0.5 to 0.)
    pub fn optimized(envelope: &[f64]) -> Vec<i8> {
        envelope
            .iter()
            .map(|e| {
                if *e >= 0.5 {
                    1
                } else if *e < -0.5 {
                    -1
                } else {
                    0
                }
            })
            .collect()
    }
}

/// ---------------------------------------------------------------------------
/// Instruction 7: TURBOVEC_DOT — 384-dim dot product; current() = plain dot,
/// optimized() = blocked dot (AVX2 256-bit block of 64) (1:1 python).
/// ---------------------------------------------------------------------------
pub struct TURBOVEC_DOT;

impl TURBOVEC_DOT {
    pub fn current(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// optimized(): blocked summation, block_size = 64 (python constant).
    pub fn optimized(a: &[f64], b: &[f64]) -> f64 {
        let block_size = 64;
        let mut total = 0.0;
        let mut i = 0;
        while i < a.len() {
            let hi = core::cmp::min(i + block_size, a.len());
            let mut acc = 0.0;
            for j in i..hi {
                acc += a[j] * b[j];
            }
            total += acc;
            i += block_size;
        }
        total
    }
}

/// ---------------------------------------------------------------------------
/// Instruction 6: DDR_MATCH — parallel antibody pattern matching (1:1 python:
/// current() case-sensitive contains; optimized() lowercases the text first).
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct DDR_MATCH {
    pub patterns: Vec<String>,
}

impl DDR_MATCH {
    /// python default: patterns or [''] * 1801
    pub const DEFAULT_PATTERN_COUNT: usize = 1801;

    pub fn new(patterns: Vec<String>) -> Self {
        DDR_MATCH { patterns }
    }

    /// python default constructor state: 1801 empty patterns (never match).
    pub fn new_default() -> Self {
        DDR_MATCH {
            patterns: vec![String::new(); Self::DEFAULT_PATTERN_COUNT],
        }
    }

    /// current(): sequential substring match, case-sensitive.
    pub fn current(&self, text: &str) -> Vec<u64> {
        let mut matches = Vec::new();
        for (i, pat) in self.patterns.iter().enumerate() {
            if !pat.is_empty() && text.contains(pat.as_str()) {
                matches.push(i as u64);
            }
        }
        matches
    }

    /// optimized(): same scan against lowercased text (1:1 python).
    pub fn optimized(&self, text: &str) -> Vec<u64> {
        let text_lower = text.to_lowercase();
        let mut matches = Vec::new();
        for (i, pat) in self.patterns.iter().enumerate() {
            if !pat.is_empty() && text_lower.contains(pat.as_str()) {
                matches.push(i as u64);
            }
        }
        matches
    }
}

/// ---------------------------------------------------------------------------
/// Instruction 5: RADIX_TRIE_WALK — single-cycle radix trie walk.
/// optimized() is a faithful port of the python array walk over a flat
/// transition table indexed by (node * 16 + nibble). current() walks a python
/// nested dict-of-'0'/'1'-strings trie — deep-Python body → MANUAL (None).
/// ---------------------------------------------------------------------------
pub struct RADIX_TRIE_WALK;

impl RADIX_TRIE_WALK {
    /// MANUAL: python body walks a nested dict trie (str(bit) keys) over all
    /// 256 bit positions — not representable in no_std without a map; the
    /// equivalent flat-table walk is implemented in optimized().
    pub fn current(_key: u64, _trie: u64) -> Option<u64> {
        None
    }

    /// optimized(): 8-byte × 2-nibble walk over flat_trie[node*16 + nibble];
    /// returns the terminal node index, or None on a -1 transition
    /// (1:1 python; python np.float32 values are integral → i64 is exact).
    pub fn optimized(key: u64, flat_trie: &[i64]) -> Option<u64> {
        let mut node: i64 = 0;
        for shift in (0..8).rev() {
            let byte = ((key >> (shift * 8)) & 0xFF) as i64;
            for nibble_shift in [4i64, 0i64] {
                let bit4 = (byte >> nibble_shift) & 0xF;
                let idx = node * 16 + bit4;
                let next_node = if (idx as usize) < flat_trie.len() {
                    flat_trie[idx as usize]
                } else {
                    -1
                };
                if next_node == -1 {
                    return None;
                }
                node = next_node;
            }
        }
        Some(node as u64)
    }
}

/// ---------------------------------------------------------------------------
/// Instructions 2/4/8: MAMBA3STEP, LORA_FUSED, INSTELLA_GATE.
/// python __init__ builds np.random.randn weight matrices (non-deterministic)
/// and the forward passes are numpy matmul/softmax chains; benchmark harness
/// scaffolding → honest MANUAL stubs (empty outputs, no fabricated numbers).
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct MAMBA3STEP {
    pub dim: usize,
    pub state_dim: usize,
    pub h: Vec<f64>,
}

impl MAMBA3STEP {
    pub fn new(dim: usize, state_dim: usize) -> Self {
        MAMBA3STEP {
            dim,
            state_dim,
            h: vec![0.0; state_dim],
        }
    }

    /// MANUAL: python body = self.A@self.h + self.B@x … (random numpy matrices).
    pub fn current(&self, _x: &[f64]) -> Vec<f64> {
        Vec::new()
    }

    /// MANUAL: python body = stateless single-pass matmul chain.
    pub fn optimized(&self, _x: &[f64]) -> Vec<f64> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct LORA_FUSED {
    pub dim: usize,
    pub rank: usize,
}

impl LORA_FUSED {
    pub fn new(dim: usize, rank: usize) -> Self {
        LORA_FUSED { dim, rank }
    }

    /// MANUAL: python body = x@A.T@B.T + x@base.T (random numpy matrices).
    pub fn current(&self, _x: &[f64]) -> Vec<f64> {
        Vec::new()
    }

    /// MANUAL: python body = fused matmul chain via np.dot.
    pub fn optimized(&self, _x: &[f64]) -> Vec<f64> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct INSTELLA_GATE {
    pub num_experts: usize,
}

impl INSTELLA_GATE {
    pub fn new(num_experts: usize) -> Self {
        INSTELLA_GATE { num_experts }
    }

    /// MANUAL: python body = softmax over 64 random-score experts + top-2
    /// (needs exp — no libm in the kernel).
    pub fn current(&self, _x: &[f64]) -> (u64, u64, f64) {
        (0, 0, 0.0)
    }

    /// MANUAL: python body = argpartition top-2 over random scores.
    pub fn optimized(&self, _x: &[f64]) -> (u64, u64, f64) {
        (0, 0, 0.0)
    }
}

/// ---------------------------------------------------------------------------
/// Benchmark suite helpers (1:1 python report arithmetic).
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub current_ns: f64,
    pub optimized_ns: f64,
    pub speedup: f64,
    pub ops_replaced_est: u64,
}

/// Deterministic post-processing python applies to measured timings
/// (round to 1 decimal, speedup = current/opt, ops_saved = int(current_ns/10)
/// when speedup > 0).
pub fn compute_report_metrics(name: &str, current_ns: f64, optimized_ns: f64) -> BenchmarkResult {
    let speedup = if optimized_ns > 0.0 {
        current_ns / optimized_ns
    } else {
        0.0
    };
    let ops_saved = if speedup > 0.0 {
        (current_ns / 10.0) as u64
    } else {
        0
    };
    BenchmarkResult {
        name: name.to_string(),
        current_ns: round1(current_ns),
        optimized_ns: round1(optimized_ns),
        speedup: round1(speedup),
        ops_replaced_est: ops_saved,
    }
}

/// MANUAL: python benchmark_instruction() runs warmup + perf_counter_ns timing
/// loops over arbitrary callables — timing is environment-dependent, so no
/// honest no_std port exists; use compute_report_metrics() on externally
/// measured ns values for the deterministic report arithmetic.
pub fn benchmark_instruction(name: &str) -> BenchmarkResult {
    compute_report_metrics(name, 0.0, 0.0)
}
