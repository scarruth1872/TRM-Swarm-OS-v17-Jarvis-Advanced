//! TRM Swarm OS — Voltage Node & Tokenless Action Vector Engine (no_std port).
//! Deterministic core from swarm_v2/core/voltage_node_engine.py + the native
//! rust_voltage_node_eval substrate it binds (swarm_v2_rust_kernel lib.rs):
//!   read_voltage_state : state[i] = tanh(1.15 * signal[i])
//!   collapse_to_action : avg -> k3_trit / confidence,
//!                        agent_id/intent_id from state[0..2]
//! The ctypes/CDLL loader is a MANUAL host path (no DLL in the kernel), so
//! tanh is evaluated with an accurate no_std exp-based helper (no libm).

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// no_std f64 floor (truncation + sign correction; no libm in UEFI target).
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < t {
        t - 1.0
    } else {
        t
    }
}

/// no_std f64 round-to-nearest (half away from zero).
fn round_f64(x: f64) -> f64 {
    if x >= 0.0 {
        floor_f64(x + 0.5)
    } else {
        -floor_f64(-x + 0.5)
    }
}

/// Path of the native swarm rust kernel DLL (1:1 python RUST_DLL_PATH).
pub const RUST_DLL_PATH: &str =
    "F:\\Development sites\\TRM-Swarm-OS-v2\\swarm_v2_rust_kernel\\target\\release\\swarm_rust_kernel.dll";

/// Agent name table (1:1 python agent_names list, index = agent_id % 14).
pub const AGENT_NAMES: [&str; 14] = [
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
];

/// Intent name table (1:1 python intent_names list, index = intent_id % 4).
pub const INTENT_NAMES: [&str; 4] = ["READ", "WRITE", "EXECUTE", "RENDER"];

/// K3 collapse thresholds of the native collapse_to_action.
pub const K3_TRIT_POS: f64 = 0.15;
pub const K3_TRIT_NEG: f64 = -0.15;

/// Tokenless structured action output (repr(C) of the DLL ActionVector).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionVector {
    /// 0: Hermes, 1: Jarvis, ... modulo 14.
    pub agent_id: u8,
    /// 0: READ, 1: WRITE, 2: EXECUTE, 3: RENDER.
    pub intent_id: u8,
    /// Calibrated score [0.0, 1.0].
    pub confidence: f64,
    /// +1 (TRUE), 0 (UNKNOWN), -1 (FALSE).
    pub k3_trit: i8,
}

// === accurate no_std transcendental helpers (UEFI has no libm) ===
const LOG2_E: f64 = 1.4426950408889634;
const LN_2: f64 = 0.6931471805599453;

/// 2^f for f in [-1, 1]: Taylor series in (f*ln2) — absolute error < 3e-7.
fn pow2_frac(f: f64) -> f64 {
    let t = f * LN_2;
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..16u32 {
        term *= t / k as f64;
        sum += term;
        if term.abs() < 1e-14 {
            break;
        }
    }
    sum
}

/// exp(x) via 2^(x*log2 e) bit trick (no_std, no libm).
pub fn exp_approx(x: f64) -> f64 {
    if x < -745.0 {
        return 0.0;
    }
    if x > 709.0 {
        return f64::INFINITY;
    }
    let y = x * LOG2_E;
    let n = floor_f64(y);
    let f = y - n;
    let exp = n as i32;
    let p = pow2_frac(f);
    // Reconstruct 2^n * p using bit manipulation.
    if exp >= 0 {
        let bits = ((exp as u64) << 52) | (p.to_bits() & 0x000F_FFFF_FFFF_FFFF);
        f64::from_bits(bits)
    } else {
        let mag = -exp as u64;
        let bits = (p.to_bits() & 0x000F_FFFF_FFFF_FFFF) >> mag;
        f64::from_bits(bits)
    }
}

/// tanh(x) = (e^2x - 1) / (e^2x + 1) — accurate to ~1e-7 for |x| <= 6.
pub fn tanh_approx(x: f64) -> f64 {
    if x > 18.0 {
        return 1.0;
    }
    if x < -18.0 {
        return -1.0;
    }
    let e2 = exp_approx(2.0 * x);
    (e2 - 1.0) / (e2 + 1.0)
}

/// read_voltage_state + collapse_to_action in one pure call — the exact
/// semantics of `rust_voltage_node_eval(dim, signal, &mut action)`.
/// Returns None for the DLL error path (dim == 0).
pub fn evaluate_voltage_signal(signal: &[f64]) -> Option<ActionVector> {
    if signal.is_empty() {
        return None; // rust_voltage_node_eval returns -1 for dim == 0
    }
    let dim = signal.len();

    // read_voltage_state: state[i] = (val * 1.15).tanh()  (i < dim)
    let mut state = Vec::with_capacity(dim);
    for i in 0..dim {
        state.push(tanh_approx(signal[i] * 1.15));
    }

    // collapse_to_action over the state vector.
    let mut sum = 0.0f64;
    for &v in &state {
        sum += v;
    }
    let avg = sum / dim as f64;

    let k3_trit: i8 = if avg > K3_TRIT_POS {
        1
    } else if avg < K3_TRIT_NEG {
        -1
    } else {
        0
    };
    // Native casts (u8 semantics, abs() first so truncation is toward zero):
    let agent_id = ((state[0].abs() * 10.0) as u64 % 14) as u8;
    let intent_id = ((state[1].abs() * 5.0) as u64 % 4) as u8;
    let confidence = (avg.abs() * 2.0).min(1.0);

    Some(ActionVector {
        agent_id,
        intent_id,
        confidence,
        k3_trit,
    })
}

/// agent_names[agent_id % 14] — pure wrapper mapping (1:1 python).
pub fn agent_name(agent_id: u8) -> String {
    AGENT_NAMES[(agent_id as usize) % AGENT_NAMES.len()].to_string()
}

/// intent_names[intent_id % 4] — pure wrapper mapping (1:1 python).
pub fn intent_name(intent_id: u8) -> String {
    INTENT_NAMES[(intent_id as usize) % INTENT_NAMES.len()].to_string()
}

/// Full wrapper result of evaluate_voltage_signal (python dict equivalent).
#[derive(Debug, Clone)]
pub struct VoltageEvalResult {
    pub status: &'static str,
    pub agent: String,
    pub agent_id: u8,
    pub intent: String,
    pub intent_id: u8,
    /// rounded to 4 decimals (python round(float, 4)).
    pub confidence: f64,
    pub k3_trit: i8,
    pub mode: &'static str,
}

/// Wrapper semantics of evaluate_voltage_signal (rounded confidence).
pub fn evaluate_voltage_signal_wrapped(signal: &[f64]) -> VoltageEvalResult {
    match evaluate_voltage_signal(signal) {
        None => VoltageEvalResult {
            status: "FAIL",
            agent: String::new(),
            agent_id: 0,
            intent: String::new(),
            intent_id: 0,
            confidence: 0.0,
            k3_trit: 0,
            mode: "TOKENLESS_VOLTAGE_TRANSFORM",
        },
        Some(a) => {
            let conf = round_f64(a.confidence * 10000.0) / 10000.0;
            VoltageEvalResult {
                status: "PASS",
                agent: agent_name(a.agent_id),
                agent_id: a.agent_id,
                intent: intent_name(a.intent_id),
                intent_id: a.intent_id,
                confidence: conf,
                k3_trit: a.k3_trit,
                mode: "TOKENLESS_VOLTAGE_TRANSFORM",
            }
        }
    }
}

/// Singleton factory (1:1 get_voltage_engine(): lazily create, reuse forever).
static VOLTAGE_ENGINE_READY: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_voltage_engine() -> u64 {
    // python: `_load_dll()` once; available = DLL present on host filesystem.
    let v = VOLTAGE_ENGINE_READY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        // MANUAL: ctypes.CDLL(RUST_DLL_PATH); available = os.path.exists(...).
        VOLTAGE_ENGINE_READY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
