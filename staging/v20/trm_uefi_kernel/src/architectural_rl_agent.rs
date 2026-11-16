//! TRM Swarm OS - architectural_rl_agent (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/architectural_rl_agent.py.
//! Functions marked // MANUAL need human verification of math semantics.
//!
//! ORACLE RECONCILIATION NOTES (against the 133-line oracle, whose native strip
//! swarm_v2_artifacts/native_voltage_strips/architectural_rl_agent.rs exports only
//! a generic TurboVec ternary dot-product kernel - _execute/_monad_commit/_phase_info -
//! and implements NONE of the reward or selection logic, so the Python fallback IS
//! the oracle and the strip is not a reference for anything below):
//!
//!  * PORTED here: evaluate_reward() and select_action() (deterministic argmax
//!    policy), the 4-row candidate action table, the agent fields
//!    (history/current_reward/learning_rate/discount_factor) and the pydantic
//!    field defaults of PolicyState.
//!  * NOT PORTED - host-side: the `logging` logger.info() line, the `time.time()`
//!    timestamp stored with every history entry, pydantic `model_dump()`
//!    serialization, and the module-level `_rl_agent: Optional[...]` singleton
//!    object (only the lazy-init flag survives in get_architectural_rl_agent).
//!  * RANDOMNESS: the oracle's select_action docstring says "Epsilon-greedy", but
//!    the executed code contains NO RNG call (`random` is imported and never
//!    used) - selection is a deterministic argmax over 4 fixed candidates with a
//!    strict `>` tie-break (first candidate wins ties). Nothing is unportable
//!    here. Had the oracle actually sampled epsilon-randomly, the firmware could
//!    not reproduce the draw: this module has no entropy source.
//!  * no_std: sqrt/exp/ln/powf are not needed (and are std-only anyway).
//!    CPython `round(x, 3)` is round-half-to-even on the EXACT binary value, so
//!    the usual `(x*1000.0).round()/1000.0` idiom is WRONG twice over (it rounds
//!    half away from zero, and the x*1000.0 scaling is itself inexact - it moves
//!    1.2345 to 1.235 where CPython gives 1.234, and overflows to inf at 1e306).
//!    round_3() below therefore formats the exact decimal expansion with alloc's
//!    `format!` and re-parses it, reproducing CPython's round() bit-for-bit.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  PolicyState 
#[derive(Debug, Clone)]
pub struct PolicyState {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub active_nodes: i64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
}

impl PolicyState {
    pub fn new() -> Self {
        // Pydantic field defaults from the oracle: cpu 20.0, memory 40.0,
        // nodes 12, latency 15.0, error_rate 0.01.
        PolicyState {
            cpu_percent: 20.0,
            memory_percent: 40.0,
            active_nodes: 12,
            avg_latency_ms: 15.0,
            error_rate: 0.01,
        }
    }
}

//  ArchitecturalAction 
/// Port of the oracle's `parameters: Dict[str, Any]` values (ints, floats and
/// the string "qiae_quantum" all appear in the candidate table).
#[derive(Debug, Clone, PartialEq)]
pub enum ActionParamValue {
    Int(i64),
    Float(f64),
    Text(&'static str),
}

#[derive(Debug, Clone)]
pub struct ArchitecturalAction {
    pub action_type: String,
    pub parameters: Vec<(String, ActionParamValue)>,
    pub expected_reward: f64,
}

impl ArchitecturalAction {
    pub fn new() -> Self {
        ArchitecturalAction {
            action_type: String::new(),
            parameters: vec![],
            expected_reward: 0.0,
        }
    }
}

//  get_architectural_rl_agent() 
static SINGLETON_ARCHITECTURAL_RL_AGENT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_architectural_rl_agent() -> u64 {
    // Singleton handle (lazy-init pattern from architectural_rl_agent.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_ARCHITECTURAL_RL_AGENT.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ARCHITECTURAL_RL_AGENT.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// =============================================================================
// Port of ArchitecturalRLAgent.evaluate_reward / .select_action  // MANUAL math
// =============================================================================

/// CPython `min(a, b)`: returns `b` only when `b < a`, so a NaN first argument
/// propagates (Rust's f64::min would instead return the non-NaN operand).
#[inline]
fn py_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// CPython `max(a, b)`: returns `b` only when `b > a`; NaN first argument
/// propagates (max(0.0, NaN) == 0.0, max(NaN, 0.0) == NaN, same as CPython).
#[inline]
fn py_max(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// CPython `round(x, 3)` - round-half-to-even applied to the exact binary value.
/// Non-finite input is returned unchanged (round(inf,3)==inf, round(nan,3)==nan).
pub fn round_3(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    match format!("{:.3}", x).parse::<f64>() {
        Ok(v) => v,
        Err(_) => x,
    }
}

//  evaluate_reward() 
/// Multi-objective reward function (oracle docstring):
/// Reward = + Throughput/Efficiency - (0.4 * Latency) - (0.5 * Resource_Spike) - (1.0 * Error_Rate)
pub fn evaluate_reward(state: &PolicyState, action: &ArchitecturalAction) -> f64 {
    let latency_penalty = py_min(state.avg_latency_ms / 100.0, 2.0);
    let cpu_penalty = py_max(0.0, (state.cpu_percent - 80.0) / 20.0);
    let error_penalty = state.error_rate * 10.0;

    let base_reward = 1.0;
    let mut reward = base_reward - (0.4 * latency_penalty) - (0.5 * cpu_penalty) - (1.0 * error_penalty);

    // Action specific bonuses. Both thresholds are STRICT `>` (cpu > 70,
    // latency > 50) and `elif` means at most one bonus is ever applied.
    if action.action_type.as_str() == "adjust_token_rate" && state.cpu_percent > 70.0 {
        reward += 0.5;
    } else if action.action_type.as_str() == "optimize_routing" && state.avg_latency_ms > 50.0 {
        reward += 0.6;
    }

    round_3(reward)
}

//  select_action() 
/// Deterministic argmax of evaluate_reward over the oracle's 4 candidates.
/// Stateless core: see ArchitecturalRLAgent::select_action for the history /
/// current_reward side effects. Every candidate is stamped with its reward
/// before the comparison (oracle order), so the returned action always carries
/// its own rounded reward - which is NOT always the oracle's `best_reward`
/// (see select_action_with_best_reward).
pub fn select_action(state: &PolicyState) -> ArchitecturalAction {
    select_action_with_best_reward(state).0
}

/// Same argmax, but also returns the oracle's local `best_reward`.
/// The distinction matters: `select_action` returns `best_action`, which may be
/// candidate #0 that never beat the -999.0 sentinel (all four rewards <= -999.0
/// is reachable - cpu_percent is uncapped in cpu_penalty), and in that case the
/// oracle still stamps `best_action.expected_reward = r` while `current_reward`
/// keeps -999.0. Both numbers are needed for an exact port.
pub fn select_action_with_best_reward(state: &PolicyState) -> (ArchitecturalAction, f64) {
    let mut candidate_actions = vec![
        ArchitecturalAction {
            action_type: String::from("adjust_token_rate"),
            parameters: vec![(String::from("token_limit"), ActionParamValue::Int(200))],
            expected_reward: 0.0,
        },
        ArchitecturalAction {
            action_type: String::from("optimize_routing"),
            parameters: vec![(
                String::from("algorithm"),
                ActionParamValue::Text("qiae_quantum"),
            )],
            expected_reward: 0.0,
        },
        ArchitecturalAction {
            action_type: String::from("rebalance_memory"),
            parameters: vec![(String::from("gc_threshold"), ActionParamValue::Float(0.85))],
            expected_reward: 0.0,
        },
        ArchitecturalAction {
            action_type: String::from("scale_nodes"),
            parameters: vec![(
                String::from("target_capacity"),
                ActionParamValue::Int(state.active_nodes + 1),
            )],
            expected_reward: 0.0,
        },
    ];

    let mut best_reward = -999.0;
    // The oracle aliases `best_action = candidate_actions[0]`; tracking the index
    // (not a pre-loop clone) is what keeps candidate #0's stamped
    // expected_reward visible when no candidate ever beats the sentinel.
    let mut best_index = 0usize;

    for i in 0..candidate_actions.len() {
        let r = evaluate_reward(state, &candidate_actions[i]);
        // The oracle stamps EVERY candidate before comparing.
        candidate_actions[i].expected_reward = r;
        // STRICT `>`: the first candidate wins any tie (oracle tie-break).
        if r > best_reward {
            best_reward = r;
            best_index = i;
        }
    }

    (candidate_actions[best_index].clone(), best_reward)
}

//  ArchitecturalRLAgent 
/// One row of `ArchitecturalRLAgent.history`. The oracle stores
/// {"timestamp": time.time(), "state": state.model_dump(),
///  "action": action.model_dump(), "reward": best_reward}; the timestamp key is
/// host-side and is deliberately absent here (no clock in the kernel) - pair
/// each pushed entry with the caller's own tick if one is needed.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub state: PolicyState,
    pub action: ArchitecturalAction,
    pub reward: f64,
}

#[derive(Debug, Clone)]
pub struct ArchitecturalRLAgent {
    pub history: Vec<HistoryEntry>,
    pub current_reward: f64,
    pub learning_rate: f64,
    pub discount_factor: f64,
}

impl ArchitecturalRLAgent {
    /// Port of `__init__` (learning_rate 0.1, discount_factor 0.9 - both are
    /// stored but never read by the oracle's own code path).
    pub fn new() -> Self {
        ArchitecturalRLAgent {
            history: vec![],
            current_reward: 0.0,
            learning_rate: 0.1,
            discount_factor: 0.9,
        }
    }

    /// Stateful wrapper: evaluate_reward(state, action) - no state is touched.
    pub fn evaluate_reward(&self, state: &PolicyState, action: &ArchitecturalAction) -> f64 {
        evaluate_reward(state, action)
    }

    /// Port of `select_action(self, state)`: argmax over the candidate table,
    /// then `self.current_reward = best_reward` and one history append. Note
    /// that the history row stores the oracle's local `best_reward` (-999.0
    /// when no candidate beats the sentinel), not the returned action's own
    /// stamped `expected_reward` - the two can differ, so take both from the
    /// same argmax pass.
    pub fn select_action(&mut self, state: &PolicyState) -> ArchitecturalAction {
        let (best_action, best_reward) = select_action_with_best_reward(state);

        self.current_reward = best_reward;
        self.history.push(HistoryEntry {
            state: state.clone(),
            action: best_action.clone(),
            reward: best_reward,
        });

        best_action
    }
}
