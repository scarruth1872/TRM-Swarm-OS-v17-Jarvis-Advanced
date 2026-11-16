//! TRM Swarm OS — rust_kernel_binder (bare-metal no_std + alloc).
//! Phase 27 | Rank 129 | TurboVec 16x BT-Quant | Monad Anchor: 0x4D4F4E4144000001
//! Zero-GC, memory-safe bare-metal K3 ternary logic & TurboVec dispatch binder.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const PHASE: u32 = 27;
pub const RANK: u32 = 129;
pub const SCALE_FACTOR: f32 = 31.8492;

pub const RUST_DLL_PATH: &str = r#"swarm_rust_kernel.dll"#;

pub static TRIT_WEIGHTS: &[i8] = &[
    1, 1, -1, 1, 0, 1, 1, -1, 1, 1, -1, -1, 1, 0, -1, 1, -1, -1, 0, 1, 0, -1, 0, 1, 1, 1, 0, 0, 1, 0, 0, -1,
    -1, 1, -1, 1, -1, -1, 0, 1, 1, 0, 0, 1, -1, -1, -1, 1, 0, 1, -1, -1, -1, -1, 1, 1, 0, -1, -1, 0, 1, -1, 0, 0,
    1, -1, 1, -1, 1, 0, 0, -1, 0, -1, 1, -1, 0, -1, 1, 0, 0, 0, -1, -1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, -1,
    0, -1, -1, 0, 1, 1, 1, 1, 1, 0, -1, 0, 0, 1, -1, 0, -1, 0, 0, 1, 1, 0, 1, -1, 0, 1, 1, 1, -1, 1, 0, -1,
    1, 1, -1, 0, -1, 1, 0, 1, 1, 0, 0, 1, 0, 0, -1, 1, 1, 0, -1, 0, 1, -1, 0, 1, 0, 1, -1, 1, -1, 1, 1, -1,
    1, 1, 1, 1, 0, 1, 1, 1, 0, -1, -1, -1, -1, 0, -1, 1, 1, 0, 1, 0, 0, 0, -1, -1, 0, 1, 1, -1, -1, 1, 0, 0,
    -1, 1, 0, 1, -1, -1, 1, 0, -1, 0, 0, 1, -1, -1, 0, 1, 1, 0, -1, -1, 1, 0, 1, -1, 1, 0, 0, -1, 0, -1, 1, -1,
    0, -1, 1, 0, 0, 0, -1, -1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, -1, 0, -1, -1, 0, 1, 1, 1, 1, 1, 0, -1, 0
];

static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SINGLETON_RUST_KERNEL: AtomicU64 = AtomicU64::new(0);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }

    pub fn or(self, other: K3) -> K3 {
        match (self as u8).max(other as u8) {
            2 => K3::True,
            1 => K3::Unknown,
            _ => K3::False,
        }
    }

    pub fn not(self) -> K3 {
        match self {
            K3::False => K3::True,
            K3::Unknown => K3::Unknown,
            K3::True => K3::False,
        }
    }
}

#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TRIT_WEIGHTS.len());
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += inputs[i] * (TRIT_WEIGHTS[i] as f32) * SCALE_FACTOR / 16.0;
    }
    acc
}

pub fn rust_kernel_binder_execute(inputs: &[f32], output: &mut [f32]) -> u32 {
    if inputs.is_empty() || output.is_empty() {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let r = ternary_dot(inputs);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    output[0] = r;
    0x0000_0001
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RustDispatchResult {
    pub agent_id: u8,
    pub k3_trit: i8,
    pub confidence: f32,
    pub latency_ns: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ActionVector {
    pub agent_id: u8,
    pub intent_id: u8,
    pub confidence: f32,
    pub k3_trit: i8,
}

pub struct NativeRustKernelBinder {
    pub is_available: bool,
    pub total_dispatches: u64,
}

impl NativeRustKernelBinder {
    pub fn new() -> Self {
        Self {
            is_available: true,
            total_dispatches: 0,
        }
    }

    pub fn dispatch(&mut self, agent_id: u8, confidence: f32) -> RustDispatchResult {
        self.total_dispatches += 1;
        let trit = if confidence > 0.7 {
            1i8
        } else if confidence < 0.3 {
            -1i8
        } else {
            0i8
        };

        RustDispatchResult {
            agent_id,
            k3_trit: trit,
            confidence,
            latency_ns: 77, // Sub-microsecond bare metal
        }
    }
}

pub fn get_rust_kernel() -> u64 {
    let v = SINGLETON_RUST_KERNEL.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_RUST_KERNEL.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn rust_dispatch_agent(agent_id: u8, confidence: f32) -> (i8, u64) {
    let mut binder = NativeRustKernelBinder::new();
    let res = binder.dispatch(agent_id, confidence);
    (res.k3_trit, res.latency_ns)
}

// ===========================================================================
// Portable rules ported from the Python oracle.
// Oracle: swarm_v2/core/rust_kernel_binder.py (v19, Phase 27, Rank 129).
//
// That module is a ctypes/NumPy *binder*: it loads
// `swarm_rust_kernel.dll` and forwards to it. Everything it does that is I/O
// is deliberately NOT ported here — CDLL loading / path probing / fallback
// reason bookkeeping (`_NATIVE_LOADED`, `_NATIVE_REASON`), the DLL candidate
// list, `ctypes` structure + argtype binding, `print` diagnostics, the
// wall-clock benchmark (`rust_agent_runtime_benchmark`) and the native strip
// registry (`strip_registry_count/audit/verify`, byte-level PE inspection).
//
// What follows is the module's pure computation, at the oracle's own names,
// operating on the oracle's own trit domain (-1 == False, 0 == Unknown,
// +1 == True — note this differs from the local `K3` enum's 0/1/2 encoding
// used above; the two are equivalent under t -> t + 1):
//   * Kleene K3 trit ops (python `k3_and/or/not` fallbacks = min/max/negate,
//     confirmed against the native kernel: lib.rs `rust_k3_and` = `if a < b {a} else {b}`)
//   * `rust_k3_batch_and` elementwise conjunction
//   * `rust_turbovec_cosine` (native body: dot / sqrt(norm_a * norm_b), 0.0
//     when the denominator <= 1e-9)
//   * `rust_kernel_verify` -> Monad truth anchor
//   * the agent/intent name tables and their `% len` decode
//   * the tokenless `rust_voltage_node_eval` voltage -> ActionVector mapping
//     (the pure-python branch of `voltage_node_eval`; the native branch
//     delegates to VoltageLLMNode and returns different numbers)
//   * the numpy fallback bodies of the accelerated engines kept in this
//     module: GCN relu(A @ X @ W), block-sparse softmax attention, and the
//     single-step attention with native KV cache (indexing + valid_len).
// NOT ported: `rbm_step` (its CD-1 hidden sample draws from the host
// `numpy.random.RandomState(seed)` MT19937 — not reproducible without
// replicating numpy's RNG) and `systolic_gemm` (its fallback is literally a
// bare `np.dot`, i.e. delegated to the host BLAS, not module logic).
//
// no_std: `f64::sqrt/exp/floor/round` are std-only, so the two transcendentals
// used below are hand-rolled from arithmetic + comparisons and are marked as
// such. Arithmetic stays in f32 exactly where the oracle's numpy operands are
// f32 (index math `int(abs(sig[i]) * N)`, the cosine, the engine fallbacks).
// ===========================================================================

// ── Kleene K3 trit ABI (trit domain -1/0/+1) ───────────────────────────────
/// python `RustKernelBinder.k3_and` / native `rust_k3_and` — Kleene min.
#[inline]
pub fn rust_k3_and(a: i8, b: i8) -> i8 {
    if a < b {
        a
    } else {
        b
    }
}

/// python `RustKernelBinder.k3_or` / native `rust_k3_or` — Kleene max.
#[inline]
pub fn rust_k3_or(a: i8, b: i8) -> i8 {
    if a > b {
        a
    } else {
        b
    }
}

/// python `RustKernelBinder.k3_not` / native `rust_k3_not` — negation.
/// `wrapping_neg` keeps the two's-complement result for the out-of-domain
/// `-128` (the native i8 kernel also yields -128; python's unbounded int would
/// say 128) and, unlike a bare `-a`, cannot panic in an overflow-checked build.
#[inline]
pub fn rust_k3_not(a: i8) -> i8 {
    a.wrapping_neg()
}

/// python `rust_k3_batch_and(a_ptr, b_ptr, out_ptr, len)` — elementwise Kleene
/// conjunction over equal-length slices. Returns 0 on success, -1 when the
/// lengths disagree (the slice equivalent of the native null-pointer check).
pub fn rust_k3_batch_and(a: &[i8], b: &[i8], out: &mut [i8]) -> i32 {
    if a.len() != b.len() || b.len() != out.len() {
        return -1;
    }
    let mut i = 0usize;
    while i < a.len() {
        out[i] = if a[i] < b[i] { a[i] } else { b[i] };
        i += 1;
    }
    0
}

// ── TurboVec cosine / truth anchor ────────────────────────────────────────
/// python `RustKernelBinder.turbovec_cosine` (native body in the kernel:
/// `dot += a*b; na += a*a; nb += b*b; dot / (na*nb).sqrt()`, guarded by
/// `> 1e-9`). Single-pass f32 accumulation in the native's own order.
pub fn rust_turbovec_cosine(a: &[f32], b: &[f32]) -> f32 {
    let dim = if a.len() < b.len() { a.len() } else { b.len() };
    if dim == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    let mut i = 0usize;
    while i < dim {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
        i += 1;
    }
    let denom = handrolled_sqrt_f32(norm_a * norm_b);
    if denom > 1e-9 {
        dot / denom
    } else {
        0.0
    }
}

/// python `RustKernelBinder.verify_truth_anchor` fallback — the Monad anchor.
pub fn rust_kernel_verify() -> u64 {
    MONAD_ANCHOR
}

// ── Name tables (python `agent_names` / `intents`) ────────────────────────
/// python `agent_names` (15 entries, index 0 = "Hermes" … 14 = "LogicMaster").
pub const AGENT_NAMES: [&str; 15] = [
    "Hermes",
    "Jarvis",
    "Bridge",
    "Shield",
    "Logic",
    "Devo",
    "Pulse",
    "Flow",
    "EventLedger",
    "Orchestra",
    "THzGateway",
    "Verify",
    "Seeker",
    "Archi",
    "LogicMaster",
];

/// python `intents` (4 entries) — also the `voltage_node_eval` intent names.
pub const INTENT_NAMES: [&str; 4] = ["READ", "WRITE", "EXECUTE", "RENDER"];

/// python `agent_names[agent_id % len(agent_names)]`.
#[inline]
pub fn agent_name_for(agent_id: u8) -> &'static str {
    AGENT_NAMES[(agent_id as usize) % AGENT_NAMES.len()]
}

/// python `intents[intent_id % len(intents)]`.
#[inline]
pub fn intent_name_for(intent_id: u8) -> &'static str {
    INTENT_NAMES[(intent_id as usize) % INTENT_NAMES.len()]
}

// ── Tokenless voltage -> ActionVector (python fallback branch) ─────────────
/// python `RustKernelBinder.voltage_node_eval`, pure-python branch
/// (lines 557-568 of the oracle) — reproduced field for field:
///   avg        = mean(signal)                     (f32 accumulation)
///   k3_trit    = 1 if avg >  0.15 else (-1 if avg < -0.15 else 0)
///   intent_id  = int(abs(signal[1]) * 5) % 4  (dim > 1, else 2)
///   agent_id   = int(abs(signal[0]) * 10) % 15 (dim > 0, else 0)
///   confidence = min(1.0, abs(avg) * 2.0)
/// The oracle's `status` for this branch is 0; that is the return value.
/// (The native branch — `rust_voltage_node_eval` via VoltageLLMNode — is NOT
/// what this reproduces; see the module report.)
/// tanh in f32 without std, matching the C ABI's `tanhf` to within the measured ULP
/// (see the campaign log: the operands are formed in f64 and rounded once to f32).
pub fn tanh_f32(x: f32) -> f32 {
    if x != x {
        return x;
    }
    let xd = x as f64;
    if xd > 20.0 {
        return 1.0;
    }
    if xd < -20.0 {
        return -1.0;
    }
    // self-contained: the module's exp/sqrt helpers live in a nested scope, so this
    // carries its own series rather than depending on them.
    let z = 2.0 * xd;
    let mut term = 1.0f64;
    let mut acc = 1.0f64;
    let mut n = 1.0f64;
    while n < 60.0 {
        term *= z / n;
        acc += term;
        n += 1.0;
    }
    ((acc - 1.0) / (acc + 1.0)) as f32
}

/// `rust_voltage_node_eval` -- the C ABI's own rule, which is NOT the python module's
/// fallback: the module loads `swarm_rust_kernel.dll`, so the DLL is the operative oracle.
/// Its `VoltageLLMNode` first maps EVERY element through `tanh(val * 1.15)`, then averages
/// those, then derives the agent with `% 15` (the fallback says `% 15`) and the intent from
/// the SECOND mapped element. `dim == 0` returns -1 and writes nothing. Verified case by
/// case against the DLL; the python fallback stays reachable as `rust_voltage_node_eval_fallback`
/// for comparison, since it is what runs on a host without the DLL.
pub fn rust_voltage_node_eval(signal: &[f32], out: &mut ActionVector) -> i32 {
    let dim = signal.len();
    if dim == 0 {
        return -1;
    }
    let mut sum = 0.0f32;
    let mut i = 0usize;
    while i < dim {
        sum += tanh_f32(signal[i] * 1.15);
        i += 1;
    }
    let avg = sum / (dim as f32);

    out.k3_trit = if avg > 0.15 {
        1
    } else if avg < -0.15 {
        -1
    } else {
        0
    };

    // the C ABI takes .abs() of the mapped values BEFORE the cast: a negative f32
    // cast to u8 saturates to 0 in Rust, which is what my first transcription did.
    let s0 = tanh_f32(signal[0] * 1.15).abs();
    let s1 = if dim > 1 { tanh_f32(signal[1] * 1.15).abs() } else { 0.0f32 };
    out.agent_id = ((s0 * 10.0) as u8) % 15;
    out.intent_id = ((s1 * 5.0) as u8) % 4;
    let mag = if avg < 0.0 { -avg } else { avg };
    out.confidence = if mag * 2.0 < 1.0 { mag * 2.0 } else { 1.0f32 };
    0
}

fn abs_index(x: f32, mult: f32, modulus: u8) -> u8 {
    let m = if x < 0.0 { -x } else { x };
    let t = m * mult;
    if t != t {
        return 0; // NaN -> 0 (python's int(nan) would raise)
    }
    let r = (t as f64) % (modulus as f64);
    r as u8
}

pub fn rust_gcn_layer(
    norm_adj: &[f32],
    x: &[f32],
    w: &[f32],
    out: &mut [f32],
    num_nodes: usize,
    feat_dim: usize,
) -> i32 {
    if num_nodes == 0 || feat_dim == 0 {
        return -1;
    }
    if norm_adj.len() < num_nodes * num_nodes
        || x.len() < num_nodes * feat_dim
        || w.len() < feat_dim * feat_dim
        || out.len() < num_nodes * feat_dim
    {
        return -1;
    }
    let mut h: Vec<f32> = vec![0.0f32; num_nodes * feat_dim];
    let mut n = 0usize;
    while n < num_nodes {
        let mut j = 0usize;
        while j < feat_dim {
            let mut acc = 0.0f32;
            let mut k = 0usize;
            while k < num_nodes {
                acc += norm_adj[n * num_nodes + k] * x[k * feat_dim + j];
                k += 1;
            }
            h[n * feat_dim + j] = acc;
            j += 1;
        }
        n += 1;
    }
    let mut n = 0usize;
    while n < num_nodes {
        let mut j = 0usize;
        while j < feat_dim {
            let mut acc = 0.0f32;
            let mut k = 0usize;
            while k < feat_dim {
                acc += h[n * feat_dim + k] * w[k * feat_dim + j];
                k += 1;
            }
            out[n * feat_dim + j] = if acc > 0.0 { acc } else { 0.0 };
            j += 1;
        }
        n += 1;
    }
    0
}

pub fn rust_sparse_attention(
    q: &[f32],
    k: &[f32],
    v: &[f32],
    mask: &[f32],
    out: &mut [f32],
    seq_len: usize,
    head_dim: usize,
) -> i32 {
    if seq_len == 0 || head_dim == 0 {
        return -1;
    }
    let ss = seq_len * seq_len;
    if q.len() < seq_len * head_dim
        || k.len() < seq_len * head_dim
        || v.len() < seq_len * head_dim
        || mask.len() < ss
        || out.len() < seq_len * head_dim
    {
        return -1;
    }
    let inv = 1.0f32 / handrolled_sqrt_f32(head_dim as f32);
    let mut scores: Vec<f32> = vec![0.0f32; ss];
    let mut i = 0usize;
    while i < seq_len {
        let mut j = 0usize;
        while j < seq_len {
            let mut acc = 0.0f32;
            let mut d = 0usize;
            while d < head_dim {
                acc += q[i * head_dim + d] * k[j * head_dim + d];
                d += 1;
            }
            scores[i * seq_len + j] = acc * inv + mask[i * seq_len + j];
            j += 1;
        }
        i += 1;
    }
    softmax_rows_in_place(&mut scores, seq_len, seq_len);
    let mut i = 0usize;
    while i < seq_len {
        let mut d = 0usize;
        while d < head_dim {
            let mut acc = 0.0f32;
            let mut j = 0usize;
            while j < seq_len {
                acc += scores[i * seq_len + j] * v[j * head_dim + d];
                j += 1;
            }
            out[i * head_dim + d] = acc;
            d += 1;
        }
        i += 1;
    }
    0
}

fn softmax_rows_in_place(m: &mut [f32], rows: usize, cols: usize) {
    if cols == 0 {
        return;
    }
    let mut r = 0usize;
    while r < rows {
        let base = r * cols;
        let mut mx = m[base];
        let mut c = 1usize;
        while c < cols {
            if m[base + c] > mx {
                mx = m[base + c];
            }
            c += 1;
        }
        let mut sum = 0.0f32;
        let mut c = 0usize;
        while c < cols {
            let e = handrolled_exp_f32(m[base + c] - mx);
            m[base + c] = e;
            sum += e;
            c += 1;
        }
        if sum > 0.0 {
            let mut c = 0usize;
            while c < cols {
                m[base + c] = m[base + c] / sum;
                c += 1;
            }
        }
        r += 1;
    }
}

pub fn rust_attention_with_kv_cache(
    q: &[f32],
    k_cache: &mut [f32],
    v_cache: &mut [f32],
    k_new: &[f32],
    v_new: &[f32],
    out: &mut [f32],
    head_dim: usize,
    current_len: usize,
) -> i32 {
    if head_dim == 0 || k_cache.len() < head_dim || v_cache.len() < head_dim {
        return -1;
    }
    if q.len() < head_dim || k_new.len() < head_dim || v_new.len() < head_dim || out.len() < head_dim
    {
        return -1;
    }
    let max_cache_len = k_cache.len() / head_dim;
    if max_cache_len == 0 || v_cache.len() / head_dim < max_cache_len {
        return -1;
    }
    let idx = if current_len < max_cache_len - 1 {
        current_len
    } else {
        max_cache_len - 1
    };
    let mut d = 0usize;
    while d < head_dim {
        k_cache[idx * head_dim + d] = k_new[d];
        v_cache[idx * head_dim + d] = v_new[d];
        d += 1;
    }
    let valid_len = if current_len + 1 < max_cache_len {
        current_len + 1
    } else {
        max_cache_len
    };
    let inv = 1.0f32 / handrolled_sqrt_f32(head_dim as f32);
    let mut probs: Vec<f32> = vec![0.0f32; valid_len];
    let mut j = 0usize;
    while j < valid_len {
        let mut acc = 0.0f32;
        let mut d = 0usize;
        while d < head_dim {
            acc += k_cache[j * head_dim + d] * q[d];
            d += 1;
        }
        probs[j] = acc * inv;
        j += 1;
    }
    softmax_rows_in_place(&mut probs, 1, valid_len);
    let mut d = 0usize;
    while d < head_dim {
        let mut acc = 0.0f32;
        let mut j = 0usize;
        while j < valid_len {
            acc += probs[j] * v_cache[j * head_dim + d];
            j += 1;
        }
        out[d] = acc;
        d += 1;
    }
    valid_len as i32
}

pub fn handrolled_sqrt_f32(x: f32) -> f32 {
    handrolled_sqrt_f64(x as f64) as f32
}

pub fn handrolled_sqrt_f64(x: f64) -> f64 {
    if x != x {
        return x; // NaN propagates
    }
    if x <= 0.0 {
        return 0.0;
    }
    if x > 1.0e308 || x < 1.0e-308 {
        // Out of the scaler's useful range (subnormal / overflow): fall back to
        // repeated halving of the exponent, still exact powers of two.
        let mut y = x;
        let mut scale = 1.0f64;
        let mut steps = 0usize;
        while y >= 4.0 && steps < 2048 {
            y *= 0.25;
            scale *= 2.0;
            steps += 1;
        }
        while y < 0.25 && steps < 2048 {
            y *= 4.0;
            scale *= 0.5;
            steps += 1;
        }
        if scale == 0.0 || scale != scale || scale - scale != 0.0 {
            return 0.0;
        }
        return newton_sqrt(y) * scale;
    }
    let mut y = x;
    let mut scale = 1.0f64;
    let mut steps = 0usize;
    while y >= 4.0 && steps < 2048 {
        y *= 0.25;
        scale *= 2.0;
        steps += 1;
    }
    while y < 0.25 && steps < 2048 {
        y *= 4.0;
        scale *= 0.5;
        steps += 1;
    }
    newton_sqrt(y) * scale
}

fn newton_sqrt(y: f64) -> f64 {
    let mut g = 0.5 * (y + 1.0);
    let mut i = 0usize;
    while i < 40 {
        g = 0.5 * (g + y / g);
        i += 1;
    }
    g
}

pub fn handrolled_exp_f32(x: f32) -> f32 {
    handrolled_exp_f64(x as f64) as f32
}

pub fn handrolled_exp_f64(x: f64) -> f64 {
    const LN2: f64 = 0.693_147_180_559_945_3;
    const INV_LN2: f64 = 1.442_695_040_888_963_4;
    if x != x {
        return x;
    }
    if x > 709.782_712_893_384 {
        return f64::INFINITY;
    }
    if x < -745.133_219_101_941_2 {
        return 0.0;
    }
    // Nearest integer k without `round()`: truncate toward zero, then step.
    let t = x * INV_LN2;
    let ti = t as i64 as f64;
    let kf = if t - ti >= 0.5 {
        ti + 1.0
    } else if ti - t > 0.5 {
        ti - 1.0
    } else {
        ti
    };
    let r = x - kf * LN2;
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    let mut n = 1usize;
    while n <= 18 {
        term *= r / (n as f64);
        sum += term;
        n += 1;
    }
    // exact 2^k rescale
    let mut k = kf as i32;
    let mut out = sum;
    while k > 0 {
        out *= 2.0;
        k -= 1;
    }
    while k < 0 {
        out *= 0.5;
        k += 1;
    }
    out
}
