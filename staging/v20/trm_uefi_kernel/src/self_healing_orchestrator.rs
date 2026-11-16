//! TRM Swarm OS — Self-Healing Orchestrator (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/self_healing_orchestrator.py & strip_self_healing.dll.
//! Tracks real-time task latencies and error rates, detects degraded agents,
//! and dynamically reassigns critical cognitive roles until nodes recover.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const HEALTH_THRESHOLD: f32 = 0.35;
pub const LATENCY_CEILING_S: f32 = 8.0;
pub const RECOVERY_STREAK_REQUIRED: u32 = 5;
pub const WINDOW_SIZE: usize = 32;

// --- AGENT METRICS ---
#[derive(Debug, Clone)]
pub struct AgentHealthTracker {
    pub role: String,
    pub task_count: u32,
    pub error_count: u32,
    pub latency_window: [f32; WINDOW_SIZE],
    pub window_ptr: usize,
    pub window_fill: usize,
    pub consecutive_successes: u32,
    pub is_degraded: bool,
    pub degraded_at_epoch_s: Option<u64>,
    pub fallback_assigned: Option<String>,
}

impl AgentHealthTracker {
    pub fn new(role: &str) -> Self {
        Self {
            role: role.to_string(),
            task_count: 0,
            error_count: 0,
            latency_window: [0.0f32; WINDOW_SIZE],
            window_ptr: 0,
            window_fill: 0,
            consecutive_successes: 0,
            is_degraded: false,
            degraded_at_epoch_s: None,
            fallback_assigned: None,
        }
    }

    pub fn record_task(&mut self, latency_s: f32, success: bool, epoch_s: u64) -> bool {
        self.task_count += 1;
        self.latency_window[self.window_ptr] = latency_s;
        self.window_ptr = (self.window_ptr + 1) % WINDOW_SIZE;
        if self.window_fill < WINDOW_SIZE {
            self.window_fill += 1;
        }

        if success {
            self.consecutive_successes += 1;
        } else {
            self.error_count += 1;
            self.consecutive_successes = 0;
        }

        let score = self.calculate_health_score();

        if score < HEALTH_THRESHOLD {
            if !self.is_degraded {
                self.is_degraded = true;
                self.degraded_at_epoch_s = Some(epoch_s);
            }
        } else if self.is_degraded && self.consecutive_successes >= RECOVERY_STREAK_REQUIRED {
            // Recovered
            self.is_degraded = false;
            self.degraded_at_epoch_s = None;
            self.fallback_assigned = None;
        }

        self.is_degraded
    }

    pub fn calculate_health_score(&self) -> f32 {
        if self.window_fill == 0 {
            return 1.0;
        }

        let mut sum_latency = 0.0f32;
        for i in 0..self.window_fill {
            sum_latency += self.latency_window[i];
        }
        let avg_latency = sum_latency / (self.window_fill as f32);

        let error_rate = if self.task_count > 0 {
            (self.error_count as f32) / (self.task_count as f32)
        } else {
            0.0
        };

        let latency_penalty = (avg_latency / LATENCY_CEILING_S).min(1.0);
        (1.0 - (error_rate * 0.6 + latency_penalty * 0.4)).max(0.0)
    }
}

// --- SELF-HEALING ORCHESTRATOR ---
pub struct SelfHealingOrchestrator {
    trackers: Vec<AgentHealthTracker>,
    reassignment_log: Vec<String>,
}

impl SelfHealingOrchestrator {
    pub fn new() -> Self {
        let council = [
            "Architect", "Logic", "Devo", "Shield", "Seeker",
            "Bridge", "Flow", "Scribe", "Validator", "Operator",
            "Sentinel", "Chronos", "Nexus", "Quantum",
        ];

        let mut trackers = Vec::with_capacity(council.len());
        for &role in &council {
            trackers.push(AgentHealthTracker::new(role));
        }

        Self {
            trackers,
            reassignment_log: Vec::new(),
        }
    }

    pub fn record_task(&mut self, role: &str, latency_s: f32, success: bool, epoch_s: u64) -> bool {
        if let Some(t) = self.trackers.iter_mut().find(|t| t.role == role) {
            t.record_task(latency_s, success, epoch_s)
        } else {
            false
        }
    }

    pub fn get_health_score(&self, role: &str) -> f32 {
        if let Some(t) = self.trackers.iter().find(|t| t.role == role) {
            t.calculate_health_score()
        } else {
            1.0
        }
    }

    pub fn get_degraded_agents(&self) -> Vec<String> {
        self.trackers.iter().filter(|t| t.is_degraded).map(|t| t.role.clone()).collect()
    }

    pub fn reassign_role(&mut self, degraded_role: &str, fallback_role: &str, epoch_s: u64) -> bool {
        if let Some(t) = self.trackers.iter_mut().find(|t| t.role == degraded_role) {
            t.fallback_assigned = Some(fallback_role.to_string());
            let log_msg = format!("Reassigned {} -> {} at epoch {}", degraded_role, fallback_role, epoch_s);
            if self.reassignment_log.len() >= 64 {
                self.reassignment_log.remove(0);
            }
            self.reassignment_log.push(log_msg);
            true
        } else {
            false
        }
    }

    pub fn get_reassignment_log(&self) -> &[String] {
        &self.reassignment_log
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_SELF_HEALING_ORCHESTRATOR: AtomicU64 = AtomicU64::new(0);

pub fn get_self_healing_orchestrator() -> u64 {
    let v = SINGLETON_SELF_HEALING_ORCHESTRATOR.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_HEALING_ORCHESTRATOR.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ============================================================================
// ORACLE-FAITHFUL ADDITIVE PORT — swarm_v2/core/self_healing_orchestrator.py
// (class SelfHealingOrchestrator).
//
// The block above is an independent model: WINDOW_SIZE 32, health =
// 1 - (error_rate*0.6 + latency_penalty*0.4), no CPU term, empty window => 1.0,
// degradation latched at record time. The oracle instead scores
//   health = (1 - latency_norm)*0.4 + (1 - error_rate)*0.3 + (1 - cpu_load)*0.3
// over a 50-sample window, with avg_latency defaulting to 0.5 on an empty window
// and cpu_load defaulting to 0.3, and it degrades strictly on health < 0.35 via a
// separate evaluate-all sweep. Those rules are absent from the block above, so
// they are added here — under the oracle's own constant and method names — and
// nothing above is rewritten.
//
// Reachability of the two oracle paths: `_evaluate_all_agents` prefers the native
// accelerator `_SELF_HEALING_DLL.strip_evaluate_all_agents`, but
// swarm_v2_artifacts/native_voltage_strips/strip_self_healing.dll exports only
// `strip_batch_scan_security` (its .rs is a generic SQL/XSS/path scanner with none
// of this module's rules), so the CDLL attribute probe raises AttributeError,
// `_SELF_HEALING_DLL` stays None and the pure-Python fallback is the only live
// path. That fallback is what is ported here.
//
// Host-side and therefore NOT ported: asyncio context manager/loop/sleep,
// time.monotonic latency sampling, wall-clock datetime.now() timestamps, logging,
// AgentMesh node lookups + specialties mutation + `_save_state`, engine_team
// provisioning, and the DLL/numpy marshalling path.
//
// Precision: the oracle computes in Python float (f64); these helpers do the same
// in f64 so scores are bit-comparable, and only the `f32` accessor narrows.
// ============================================================================

use alloc::collections::VecDeque;

/// Instrumentation + health scoring for the agent roles, mirroring the oracle's
/// five state containers one-for-one.
///
/// * `windows`      <- `_latency_windows` (ordered map: registration order is the
///                     output order of `_evaluate_all_agents`)
/// * `task_counts`  <- `_task_counts`   (missing role reads as 0)
/// * `error_counts` <- `_error_counts`  (missing role reads as 0)
/// * `cpu_loads`    <- `_cpu_loads`     (missing role reads as DEFAULT_CPU_LOAD)
/// * `degraded`     <- `_degraded_agents` membership -> `recovery_streak`
#[derive(Debug, Clone, Default)]
pub struct OracleHealthRegistry {
    windows: Vec<(String, VecDeque<f64>)>,
    task_counts: Vec<(String, u32)>,
    error_counts: Vec<(String, u32)>,
    cpu_loads: Vec<(String, f64)>,
    degraded: Vec<(String, u32)>,
}

impl OracleHealthRegistry {
    // --- CONSTANTS, verbatim from the oracle class body ---
    pub const HEALTH_THRESHOLD: f64 = 0.35;
    pub const LATENCY_CEILING: f64 = 8.0; // seconds
    pub const RECOVERY_STREAK_REQUIRED: u32 = 5;
    pub const WINDOW_SIZE: usize = 50; // deque(maxlen=WINDOW_SIZE)
    /// `avg_latency` when the latency window is empty (or the role is unknown).
    pub const DEFAULT_AVG_LATENCY: f64 = 0.5;
    /// `self._cpu_loads.get(agent_role, 0.3)`.
    pub const DEFAULT_CPU_LOAD: f64 = 0.3;
    pub const LATENCY_WEIGHT: f64 = 0.4;
    pub const ERROR_WEIGHT: f64 = 0.3;
    pub const CPU_WEIGHT: f64 = 0.3;
    /// `LATENCY_CEILING * 0.6` — the healthy bar used by the recovery streak.
    pub const HEALTHY_LATENCY_FRACTION: f64 = 0.6;

    pub fn new() -> Self {
        Self::default()
    }

    // --- state accessors (oracle dict reads, with the oracle's own defaults) ---

    pub fn latency_window(&self, role: &str) -> Option<&VecDeque<f64>> {
        self.windows.iter().find(|(r, _)| r == role).map(|(_, w)| w)
    }

    pub fn task_count(&self, role: &str) -> u32 {
        self.task_counts
            .iter()
            .find(|(r, _)| r == role)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    }

    pub fn error_count(&self, role: &str) -> u32 {
        self.error_counts
            .iter()
            .find(|(r, _)| r == role)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    }

    pub fn cpu_load(&self, role: &str) -> f64 {
        self.cpu_loads
            .iter()
            .find(|(r, _)| r == role)
            .map(|(_, c)| *c)
            .unwrap_or(Self::DEFAULT_CPU_LOAD)
    }

    pub fn is_degraded(&self, role: &str) -> bool {
        self.degraded.iter().any(|(r, _)| r == role)
    }

    pub fn recovery_streak(&self, role: &str) -> u32 {
        self.degraded
            .iter()
            .find(|(r, _)| r == role)
            .map(|(_, s)| *s)
            .unwrap_or(0)
    }

    pub fn tracked_roles(&self) -> Vec<String> {
        self.windows.iter().map(|(r, _)| r.clone()).collect()
    }

    // --- state writers ---

    /// The three `setdefault` lines at the top of `track_task`: the role becomes
    /// visible to `_evaluate_all_agents` (empty window, zero counts).
    pub fn ensure_tracked(&mut self, role: &str) {
        if !self.windows.iter().any(|(r, _)| r == role) {
            self.windows
                .push((role.to_string(), VecDeque::with_capacity(Self::WINDOW_SIZE)));
        }
        if !self.task_counts.iter().any(|(r, _)| r == role) {
            self.task_counts.push((role.to_string(), 0));
        }
        if !self.error_counts.iter().any(|(r, _)| r == role) {
            self.error_counts.push((role.to_string(), 0));
        }
    }

    /// Raw window injection, as the oracle's `_latency_windows[role] = deque(v, maxlen=50)`
    /// does: the LAST WINDOW_SIZE samples are kept.
    pub fn set_latency_window(&mut self, role: &str, values: &[f64]) {
        let mut window: VecDeque<f64> = VecDeque::with_capacity(Self::WINDOW_SIZE);
        let start = values.len().saturating_sub(Self::WINDOW_SIZE);
        for v in &values[start..] {
            window.push_back(*v);
        }
        if let Some(slot) = self.windows.iter_mut().find(|(r, _)| r == role) {
            slot.1 = window;
        } else {
            self.windows.push((role.to_string(), window));
        }
    }

    /// Direct `_task_counts[role] = n` / `_error_counts[role] = n` assignment.
    pub fn set_counts(&mut self, role: &str, task_count: u32, error_count: u32) {
        if let Some(slot) = self.task_counts.iter_mut().find(|(r, _)| r == role) {
            slot.1 = task_count;
        } else {
            self.task_counts.push((role.to_string(), task_count));
        }
        if let Some(slot) = self.error_counts.iter_mut().find(|(r, _)| r == role) {
            slot.1 = error_count;
        } else {
            self.error_counts.push((role.to_string(), error_count));
        }
    }

    /// `update_cpu_load`: writes `_cpu_loads` only. A role that has never been
    /// tracked stays invisible to `_evaluate_all_agents`.
    pub fn update_cpu_load(&mut self, role: &str, cpu_load: f64) {
        if let Some(slot) = self.cpu_loads.iter_mut().find(|(r, _)| r == role) {
            slot.1 = cpu_load;
        } else {
            self.cpu_loads.push((role.to_string(), cpu_load));
        }
    }

    /// Test/integration hook for `_reassign_role`'s write into `_degraded_agents`
    /// (`{..., "recovery_streak": 0}`). The mesh mutation itself is host-side.
    pub fn mark_degraded(&mut self, role: &str, recovery_streak: u32) {
        if let Some(slot) = self.degraded.iter_mut().find(|(r, _)| r == role) {
            slot.1 = recovery_streak;
        } else {
            self.degraded.push((role.to_string(), recovery_streak));
        }
    }

    /// `_restore_agent`'s `del self._degraded_agents[agent_role]`.
    pub fn clear_degraded(&mut self, role: &str) {
        self.degraded.retain(|(r, _)| r != role);
    }

    // --- track_task, minus host-side timing/logging/exception plumbing ---

    /// The body of the `track_task` context manager after the block exits.
    /// Returns `true` when the oracle would have called `_restore_agent`
    /// (recovery streak reached RECOVERY_STREAK_REQUIRED).
    pub fn record_task(&mut self, role: &str, latency_s: f64, success: bool) -> bool {
        self.ensure_tracked(role);

        if let Some((_, window)) = self.windows.iter_mut().find(|(r, _)| r == role) {
            if window.len() >= Self::WINDOW_SIZE {
                window.pop_front(); // deque(maxlen=WINDOW_SIZE) eviction
            }
            window.push_back(latency_s);
        }
        if let Some(slot) = self.task_counts.iter_mut().find(|(r, _)| r == role) {
            slot.1 = slot.1.wrapping_add(1);
        }

        if !success {
            // The oracle's `except` branch records the failure and re-raises,
            // so the recovery block below is skipped entirely.
            if let Some(slot) = self.error_counts.iter_mut().find(|(r, _)| r == role) {
                slot.1 = slot.1.wrapping_add(1);
            }
            return false;
        }

        if self.is_degraded(role) {
            if latency_s < Self::LATENCY_CEILING * Self::HEALTHY_LATENCY_FRACTION {
                let streak = self.recovery_streak(role).wrapping_add(1);
                if streak >= Self::RECOVERY_STREAK_REQUIRED {
                    self.clear_degraded(role); // _restore_agent: entry deleted
                    return true;
                }
                if let Some(slot) = self.degraded.iter_mut().find(|(r, _)| r == role) {
                    slot.1 = streak;
                }
            } else if let Some(slot) = self.degraded.iter_mut().find(|(r, _)| r == role) {
                slot.1 = 0;
            }
        }
        false
    }

    // --- compute_health_score ---

    /// `compute_health_score` in the oracle's own f64 arithmetic.
    ///
    /// The clamp is the oracle's `max(0.0, min(1.0, health))` written out rather
    /// than `f64::clamp`/`min`/`max`, because the built-ins disagree with CPython
    /// on NaN (CPython returns 1.0, `min`/`max` return the non-NaN operand).
    pub fn compute_health_score_f64(&self, role: &str) -> f64 {
        let avg_latency = match self.latency_window(role) {
            Some(window) if !window.is_empty() => {
                let mut sum = 0.0f64;
                for v in window.iter() {
                    sum += *v;
                }
                sum / (window.len().max(1) as f64)
            }
            _ => Self::DEFAULT_AVG_LATENCY,
        };

        // `min(avg_latency / LATENCY_CEILING, 1.0)` with CPython's association
        // order: `min(a, b)` keeps `a` unless `b < a`, so a NaN ratio propagates
        // (Rust's `f64::min` would instead return the non-NaN operand, 1.0).
        let latency_ratio = avg_latency / Self::LATENCY_CEILING;
        let normalized_latency = if 1.0 < latency_ratio { 1.0 } else { latency_ratio };

        let task_count = self.task_count(role);
        let error_count = self.error_count(role);
        let error_rate = (error_count as f64) / ((task_count.max(1)) as f64);

        let cpu_load = self.cpu_load(role);

        // Same association order as the oracle: (a*0.4 + b*0.3) + c*0.3.
        let health = (1.0 - normalized_latency) * Self::LATENCY_WEIGHT
            + (1.0 - error_rate) * Self::ERROR_WEIGHT
            + (1.0 - cpu_load) * Self::CPU_WEIGHT;

        let below_one = if health < 1.0 { health } else { 1.0 }; // Python min(1.0, health)
        if below_one > 0.0 {
            below_one
        } else {
            0.0
        } // Python max(0.0, _)
    }

    /// Same value narrowed to the module's f32 surface.
    pub fn compute_health_score(&self, role: &str) -> f32 {
        self.compute_health_score_f64(role) as f32
    }

    /// `_evaluate_all_agents` — the Python fallback branch (the DLL path is dead,
    /// see the header). Strictly `< HEALTH_THRESHOLD`, iterated in window order.
    pub fn _evaluate_all_agents(&self) -> Vec<String> {
        let mut degraded = Vec::new();
        for (role, _) in self.windows.iter() {
            if self.compute_health_score_f64(role) < Self::HEALTH_THRESHOLD {
                degraded.push(role.clone());
            }
        }
        degraded
    }

    /// The scoring half of `_reassign_role`: the healthiest peer of `degraded_role`
    /// among `engine_team`, skipping the degraded role itself and every role in
    /// `_degraded_agents`, taking the first strict maximum from a -1.0 floor.
    /// Mesh node lookup, specialty transfer and `_save_state` are host-side.
    pub fn select_reassignment_target(
        &self,
        degraded_role: &str,
        engine_team: &[String],
    ) -> Option<String> {
        let mut best_candidate: Option<String> = None;
        let mut best_score: f64 = -1.0;
        for role in engine_team.iter() {
            if role == degraded_role || self.is_degraded(role) {
                continue;
            }
            let score = self.compute_health_score_f64(role);
            if score > best_score {
                best_score = score;
                best_candidate = Some(role.clone());
            }
        }
        best_candidate
    }
}

