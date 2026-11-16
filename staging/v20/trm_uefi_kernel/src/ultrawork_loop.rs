//! TRM Swarm OS - ultrawork_loop (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/ultrawork_loop.py.
//! Functions marked // MANUAL need human verification of math semantics.
//!
//! Verified against the Python oracle swarm_v2/core/ultrawork_loop.py:
//!   * native_evaluate_mission_steps() - mission velocity + K3 trit scoring. The
//!     math is identical to swarm_v2_artifacts/native_voltage_strips/
//!     strip_ultrawork_gate.rs (strip_evaluate_missions), which the Python wrapper
//!     loads via ctypes whenever the DLL is present.
//!   * MissionPhase, k3_gate_route() - the pure routing thresholds of k3_gate().
//!   * parse_plan() - the _parse_plan() step-extraction rules.
//! Host-side, deliberately NOT ported: os/json/logging, MISSIONS_DIR file I/O
//! (_persist/_resume_mission), datetime.now() timestamps, the asyncio
//! planner/executor/verifier arguments, and the Kleene engine / DigitalTwin /
//! GPU-fabric / hardware-binder imports plus their singleton handles.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MISSIONS_DIR: u64 = 0;

// Kleene trits (ultrawork_loop.py imports TRUE/FALSE/UNKNOWN from
// ternary_kleene_engine: TRUE = 1, FALSE = -1, UNKNOWN = 0).
pub const TRUE: i32 = 1;
pub const FALSE: i32 = -1;
pub const UNKNOWN: i32 = 0;

//  MissionPhase (ultrawork_loop.py, class MissionPhase(Enum)) 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionPhase {
    Planning,
    Acting,
    Verifying,
    Completed,
    Failed,
    FastPath,
}

impl MissionPhase {
    /// Python `MissionPhase.<variant>.value`.
    pub fn value(&self) -> &'static str {
        match self {
            MissionPhase::Planning => "planning",
            MissionPhase::Acting => "acting",
            MissionPhase::Verifying => "verifying",
            MissionPhase::Completed => "completed",
            MissionPhase::Failed => "failed",
            MissionPhase::FastPath => "fast_path", // K3-decidable: executed natively, no LLM
        }
    }
}

//  MissionStep 
#[derive(Debug, Clone)]
pub struct MissionStep {
    pub description: String,
    pub acceptance_criteria: String,
    pub status: String,
    pub result: String,
    pub attempts: i64,
}

impl MissionStep {
    pub fn new() -> Self {
        MissionStep {
            description: String::new(),
            acceptance_criteria: String::new(),
            // Python dataclass default: status="pending" (pending/running/completed/failed).
            status: String::from("pending"),
            result: String::new(),
            attempts: 0,
        }
    }
}

//  MissionState 
#[derive(Debug, Clone)]
pub struct MissionState {
    pub mission_id: String,
    pub objective: String,
    pub phase: String,
    pub steps: Vec<MissionStep>,
    pub plan_text: String,
    pub progress_log: Vec<String>,
    pub verification_result: String,
    pub retry_count: i64,
    pub max_retries: i64,
    pub created_at: String,
    pub completed_at: String,
    // K3 gate decision (v19.010+)
    pub gate: String,          // K3_FAST_PATH | INSTELLA_MOE | BONSAI_TERNARY | K3_REJECT | FULL_CYCLE
    pub gate_trit: i32,        // -1, 0, +1
    pub gate_confidence: f64,
    pub gate_ambiguity: f64,
    pub gate_agent_slot: i64,
    pub gate_latency: String,
    pub llm_avoided: bool,
}

impl MissionState {
    pub fn new() -> Self {
        MissionState {
            mission_id: String::new(),
            objective: String::new(),
            phase: String::from("planning"),
            steps: vec![],
            plan_text: String::new(),
            progress_log: vec![],
            verification_result: String::new(),
            retry_count: 0,
            max_retries: 3,
            created_at: String::new(),
            completed_at: String::new(),
            gate: String::new(),
            gate_trit: 0,
            gate_confidence: 0.0,
            gate_ambiguity: 0.0,
            gate_agent_slot: 0,
            gate_latency: String::new(),
            llm_avoided: false,
        }
    }
}

//  native_evaluate_mission_steps() 
// Python: `native_evaluate_mission_steps(statuses, attempts) -> (vels, trits)`
// (ultrawork_loop.py). statuses/attempts are row-major (n_missions, n_steps):
// status 2 == completed, 3 == failed. Returns the mission velocities and the
// per-mission K3 trit, byte-for-byte identical to strip_evaluate_missions.
// no_std: no numpy, caller passes flat slices.
pub fn native_evaluate_mission_steps(
    statuses: &[u8],
    attempts: &[u32],
    n_missions: usize,
    n_steps: usize,
) -> (Vec<f64>, Vec<i32>) {
    let mut vels: Vec<f64> = Vec::new();
    let mut trits: Vec<i32> = Vec::new();

    // Python guards the DLL path with `n_m > 0 and n_s > 0`; with n_s == 0 the
    // Python fallback raises ZeroDivisionError. no_std returns empty here.
    if n_missions == 0 || n_steps == 0 {
        return (vels, trits);
    }
    // The C strip would read out of bounds; return empty instead of panicking.
    if statuses.len() < n_missions * n_steps || attempts.len() < n_missions * n_steps {
        return (vels, trits);
    }

    let inv_steps = 1.0f64 / (n_steps as f64);

    for m in 0..n_missions {
        let base = m * n_steps;
        let mut comp: usize = 0;
        let mut fail_pen: f64 = 0.0;

        for s in 0..n_steps {
            let st = statuses[base + s];
            let att = attempts[base + s] as f64;
            if st == 2 {
                comp += 1;
            } else if st == 3 {
                fail_pen += att * 0.5f64;
            }
        }

        // v = max(0.0, min(1.0, (comp / n_s) - (fail_pen / n_s) * 0.25))
        // Written in the strip's form (comp * inv_steps) so the last-ulp result
        // matches the native strip the Python wrapper actually calls.
        let comp_ratio = (comp as f64) * inv_steps;
        let mut vel = comp_ratio - (fail_pen * inv_steps) * 0.25f64;
        if vel < 0.0 {
            vel = 0.0;
        }
        if vel > 1.0 {
            vel = 1.0;
        }

        vels.push(vel);
        // K3 gate: v >= 0.75 -> +1, v >= 0.35 -> 0, else -1 (non-strict bounds).
        trits.push(if vel >= 0.75f64 {
            1
        } else if vel >= 0.35f64 {
            0
        } else {
            -1
        });
    }

    (vels, trits)
}

//  k3_gate() routing 
// Pure routing decision table of UltraworkLoop.k3_gate (ultrawork_loop.py).
// Everything else in that method is host-side (radix trie lookup, hardware
// binder, GPU fabric, Kleene engine, DigitalTwin dispatch), so only the
// decision ladder is portable. `trie_matched` stands in for the radix-trie hit.
pub fn k3_gate_route(
    trie_matched: bool,
    trit: i32,
    ambiguity_ratio: f64,
    phi_v: f64,
) -> &'static str {
    if trie_matched {
        // Voltage-bound: fast path only when silicon is healthy.
        if phi_v < 0.5 {
            return "VOLTAGE_THROTTLE";
        }
        return "K3_FAST_PATH";
    }
    // K3 D(Psi) is used only when FULLY decisive (amb == 0.0).
    if ambiguity_ratio == 0.0 {
        if trit == TRUE {
            return "K3_FAST_PATH";
        } else if trit == FALSE {
            return "K3_REJECT";
        }
    }
    // Ambiguity routing: amb < 0.05 FAST_PATH, amb <= 0.60 INSTELLA_MOE, else BONSAI.
    if ambiguity_ratio < 0.05 {
        "K3_FAST_PATH"
    } else if ambiguity_ratio <= 0.60 {
        "INSTELLA_MOE"
    } else {
        "BONSAI_TERNARY"
    }
}

//  k3_gate() exception fallback 
// Python: except branch returns {"gate": "FULL_CYCLE", "trit": 0,
// "confidence": 0.0, "ambiguity_ratio": 0.5, "llm_avoided": False}.
pub fn k3_gate_fallback() -> (&'static str, i32, f64, f64, bool) {
    ("FULL_CYCLE", UNKNOWN, 0.0, 0.5, false)
}

//  _parse_plan() 
/// Python `str.isspace()` predicate: the Unicode White_Space property plus the
/// U+001C..U+001F separators (Rust's `char::is_whitespace` omits those four).
fn py_isspace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1C}'..='\u{1F}')
}

/// Python `str.strip()`.
fn py_strip(s: &str) -> &str {
    s.trim_matches(py_isspace)
}

/// Python `str.isdigit()` approximation. // MANUAL
/// no_std has no Unicode numeric tables; `char::is_numeric()` accepts everything
/// Python's isdigit accepts (all Nd + Digit numerals) plus the No fractions and
/// Nl roman numerals that Python's isdigit rejects, so a line led by 1/2 or XII
/// is a step here but not in Python. ASCII input is unaffected.
fn is_python_isdigit(c: char) -> bool {
    c.is_numeric()
}

/// Port of `UltraworkLoop._parse_plan(plan_text) -> List[Dict]`.
/// Step lines start with a digit, '-' or '*'; the marker prefix is then stripped
/// with the explicit character set "0123456789.-*) ". An empty result falls back
/// to a single step whose description is the first 200 CHARACTERS of plan_text
/// (Python slices characters, not bytes).
pub fn parse_plan(plan_text: &str) -> Vec<MissionStep> {
    let mut steps: Vec<MissionStep> = Vec::new();

    for raw in py_strip(plan_text).split('\n') {
        let line = py_strip(raw);
        let mut first = line.chars();
        let starts_step = match first.next() {
            Some(c) => is_python_isdigit(c) || line.starts_with('-') || line.starts_with('*'),
            None => false,
        };
        if starts_step {
            let desc = py_strip(line.trim_start_matches(|c: char| {
                matches!(c, '0'..='9' | '.' | '-' | '*' | ')' | ' ')
            }));
            if !desc.is_empty() {
                steps.push(MissionStep {
                    description: String::from(desc),
                    acceptance_criteria: String::new(),
                    status: String::from("pending"),
                    result: String::new(),
                    attempts: 0,
                });
            }
        }
    }

    if steps.is_empty() {
        // Python: [{"description": plan_text[:200], ...}] - 200 characters.
        let head: String = plan_text.chars().take(200).collect();
        steps.push(MissionStep {
            description: head,
            acceptance_criteria: String::new(),
            status: String::from("pending"),
            result: String::new(),
            attempts: 0,
        });
    }

    steps
}

//  get_ultrawork_loop() 
static SINGLETON_ULTRAWORK_LOOP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_ultrawork_loop() -> u64 {
    // Singleton handle (lazy-init pattern from ultrawork_loop.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_ULTRAWORK_LOOP.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ULTRAWORK_LOOP.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

//  execute_mission() verify gate 
// Python (ultrawork_loop.py, execute_mission VERIFY branch):
//     if "PASS" in str(verification).upper():  -> COMPLETED, else reset steps + retry.
// The rest of that method is host-side (async planner/executor/verifier callbacks,
// the K3 gate imports, file I/O); this is the one portable decision: a
// case-insensitive "PASS" substring ("pass", "Passed", "PASS: ..." all pass).
pub fn verification_passes(verification: &str) -> bool {
    let b = verification.as_bytes();
    let n = b.len();
    if n < 4 {
        return false;
    }
    for i in 0..=(n - 4) {
        if b[i].to_ascii_uppercase() == b'P'
            && b[i + 1].to_ascii_uppercase() == b'A'
            && b[i + 2].to_ascii_uppercase() == b'S'
            && b[i + 3].to_ascii_uppercase() == b'S'
        {
            return true;
        }
    }
    false
}
