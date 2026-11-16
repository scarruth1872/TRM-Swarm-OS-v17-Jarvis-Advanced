//! TRM Swarm OS — Voltage Node & Tokenless Action Vector Engine (no_std port).
//! Ported from swarm_v2/core/voltage_node_engine.py (v18 engine binder).
//!
//! The python module is a thin ctypes/NumPy bridge to a host-side DLL
//! (`rust_voltage_node_eval`): all per-sample "voltage → action vector" math
//! lives inside that native library and is NOT reproducible in a no_std UEFI
//! kernel (numpy arrays, ctypes pointers, DLL loading — none portable here).
//!
//! This port keeps the deterministic, host-side subset 1:1 with python:
//! the `CActionVector` shape, the agent/intent name tables, python's
//! `agent_id % 14` / `intent_id % 4` decoding, and the 4-decimal confidence
//! rounding used to build the `evaluate_voltage_signal` result. The raw
//! signal→vector transform itself is marked // MANUAL below.

extern crate alloc;

/// python `CActionVector` ctypes.Structure (agent_id, intent_id, confidence,
/// k3_trit). Plain value struct in no_std.
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionVector {
    pub agent_id: u8,
    pub intent_id: u8,
    pub confidence: f32,
    pub k3_trit: i8,
}

/// python `agent_names` table (index 0 = "Hermes" … 13 = "Archi").
pub const AGENT_NAMES: &[&str] = &[
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

/// python `intent_names` table.
pub const INTENT_NAMES: &[&str] = &["READ", "WRITE", "EXECUTE", "RENDER"];

/// python `agent_names[action.agent_id % len(agent_names)]`.
pub fn agent_name(agent_id: u8) -> &'static str {
    let n = AGENT_NAMES.len();
    AGENT_NAMES[(agent_id as usize) % n]
}

/// python `intent_names[action.intent_id % len(intent_names)]`.
pub fn intent_name(intent_id: u8) -> &'static str {
    let n = INTENT_NAMES.len();
    INTENT_NAMES[(intent_id as usize) % n]
}

/// Python `round(float(x), 4)` — round-half-even (banker's) to 4 decimals.
///
/// Implemented without float intrinsic methods (`f64::round`/`f64::floor` are
/// absent from this target's bin core) via IEEE-754 bit manipulation. Input is
/// an f32: f32 has a 24-bit mantissa and 10^4 = 2^4·5^4 needs at most 34 bits,
/// so `x * 10^4` is EXACT as an f64 integer — nothing is lost before the tie.
pub fn round4(x: f32) -> f64 {
    round_half_even(x as f64 * 10000.0) / 10000.0
}

/// Round half to even (python 3 `round`) over the exact-integer domain that
/// `round4` feeds it (|v| = |x·10^4| ≤ ~3.4e34, exact integer doubles). Handles
/// every normal double; sub-|1| values round to 0 or 1 with 0.5 → 0 (even).
fn round_half_even(v: f64) -> f64 {
    let bits = v.to_bits();
    let sign = bits & (1u64 << 63);
    let mag = bits & 0x7fff_ffff_ffff_ffff;
    let exp_field = (mag >> 52) as i32;
    if exp_field == 0x7ff || mag == 0 {
        return v; // NaN / ±inf / ±0 unchanged
    }
    let exp_unb = exp_field - 1023; // unbiased exponent
    // |v| < 1: rounds to 0 or 1; the only tie is exactly 0.5 → 0 (even).
    if exp_unb < 0 {
        let abs_v = f64::from_bits(mag);
        let r = if abs_v > 0.5 { 1.0 } else { 0.0 };
        return if sign != 0 { -r } else { r };
    }
    let mant = (mag & ((1u64 << 52) - 1)) | (1u64 << 52); // significand + implied bit
    let frac_bits = 52 - exp_unb; // bits strictly below the binary point
    if frac_bits <= 0 {
        return v; // |v| ≥ 2^52: already an integer
    }
    let unit = 1u64 << frac_bits; // value of the lowest integer bit, in mantissa units
    let frac = mant & (unit - 1);
    let half = unit >> 1;
    let lower = mant - frac;
    let rounded = if frac > half {
        lower + unit
    } else if frac < half {
        lower
    } else if lower & unit == 0 {
        lower // exact tie → keep even
    } else {
        lower + unit // exact tie → round up to even
    };
    // Rounding up can carry the significand into the next exponent.
    let (out_exp, out_mant) = if rounded >= (1u64 << 53) {
        (exp_unb + 1, rounded >> 1)
    } else {
        (exp_unb, rounded)
    };
    let out_mag = (((out_exp + 1023) as u64) << 52) | (out_mant & ((1u64 << 52) - 1));
    f64::from_bits(sign | out_mag)
}

/// Result dict of python `evaluate_voltage_signal` — flattened into a struct
/// (no_std has no dict); every field is produced exactly as python builds it.
#[derive(Clone, Copy, Debug)]
pub struct VoltageEvalResult {
    pub status: &'static str,
    pub agent: &'static str,
    pub agent_id: u8,
    pub intent: &'static str,
    pub intent_id: u8,
    pub confidence: f64,
    pub k3_trit: i8,
    pub mode: &'static str,
}

/// Host-side decode half of python `evaluate_voltage_signal` — given an
/// already-computed action vector it reproduces the python result dict 1:1.
///
/// // MANUAL: the *signal → action vector* transform performed by the native
/// // DLL (`rust_voltage_node_eval(dim, float_signal[], *CActionVector)`) has
/// // no python source body to port — it is a compiled host binary. On the
/// // booted kernel this must be supplied by the platform/bridge layer before
/// // calling this decoder; it is not fabricatable here.
pub fn decode_voltage_eval(action: &ActionVector) -> VoltageEvalResult {
    VoltageEvalResult {
        status: "PASS",
        agent: agent_name(action.agent_id),
        agent_id: action.agent_id,
        intent: intent_name(action.intent_id),
        intent_id: action.intent_id,
        confidence: round4(action.confidence),
        k3_trit: action.k3_trit,
        mode: "TOKENLESS_VOLTAGE_TRANSFORM",
    }
}

/// Engine handle — python `VoltageNodeEngine`. `available` mirrors python's
/// `self.available`, which is True only after the host DLL loads.
///
/// // MANUAL: a UEFI kernel cannot load the Windows `swarm_rust_kernel.dll`
/// // (python `RUST_DLL_PATH`), so `available` is a compile-time flag for the
/// // platform layer to raise when a native evaluation backend is present.
/// // While false, python would return {"status": "FAIL", "error":
/// // "DLL unavailable"} — callers should treat decode_voltage_eval the same.
#[derive(Clone, Copy, Debug)]
pub struct VoltageNodeEngine {
    pub available: bool,
}

impl VoltageNodeEngine {
    pub const fn new() -> Self {
        VoltageNodeEngine { available: false }
    }
}

/// Singleton — python `get_voltage_engine()` (module-level instance).
pub static VOLTAGE_ENGINE: VoltageNodeEngine = VoltageNodeEngine::new();

pub fn get_voltage_engine() -> &'static VoltageNodeEngine {
    &VOLTAGE_ENGINE
}
