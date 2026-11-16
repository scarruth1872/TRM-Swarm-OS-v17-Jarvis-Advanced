//! TRM Swarm OS — Cognitive Load Balancer (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/cognitive_load_balancer.py & strip_cognitive_load_balancer.dll.
//! Dynamically routes reasoning tasks across the 14 Council agents based on capability matching,
//! active backlog depth, moving average completion latency, and cognitive stress scores.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- AGENT LOAD METRICS ---
/// Per-agent load metrics.
///
/// Model corrected to the source's (`swarm_v2/core/cognitive_load_balancer.py`): the source's
/// `record_task_start` touches ONLY the pending count and the last-active stamp, and its
/// `record_task_end` SETS `current_complexity` to the completed task's complexity and recomputes the
/// average over a 50-deep history of completion times. The previous port accumulated complexity at
/// assignment and used an exponential moving average (0.8/0.2) -- a different quantity wearing the
/// same field names. One model answers the question now: the source's.
pub const HISTORY_CAP: usize = 50;

#[derive(Debug, Clone)]
pub struct AgentLoadMetrics {
    pub role: String,
    pub active_tasks: u32,
    pub completed_tasks: u64,
    /// Mean of the last `HISTORY_CAP` completion times (ms). Kept as f32 for `cognitive_load`.
    pub avg_completion_ms: f32,
    /// Source's `current_complexity`: SET at completion, never accumulated at start.
    pub current_complexity_score: f32,
    pub last_active_epoch_s: u64,
    /// Source's `_completion_history[role]`, capped at HISTORY_CAP by popping the front.
    pub completion_history: Vec<f64>,
}

impl AgentLoadMetrics {
    /// The source's `_agent_load[role]` default: `last_active` stays 0.0 until the role starts a task.
    pub fn new(role: &str) -> Self {
        Self {
            role: role.to_string(),
            active_tasks: 0,
            completed_tasks: 0,
            // The source's declared default is 0.0, not a nominal baseline.
            avg_completion_ms: 0.0,
            current_complexity_score: 0.0,
            last_active_epoch_s: 0,
            completion_history: Vec::new(),
        }
    }

    pub fn cognitive_load(&self) -> f32 {
        (self.active_tasks as f32) * 1.5 + self.current_complexity_score + (self.avg_completion_ms / 50.0)
    }

    /// Source's `record_task_start`: pending count up, last-active stamped, complexity untouched.
    pub fn assign_task(&mut self, _complexity: f32, epoch_s: u64) {
        self.active_tasks += 1;
        self.last_active_epoch_s = epoch_s;
    }

    /// Source's `record_task_end`: complexity SET, pending count down, history appended and averaged.
    pub fn complete_task(&mut self, duration_ms: f32, complexity: f32, epoch_s: u64) {
        if self.active_tasks > 0 {
            self.active_tasks -= 1;
        }
        self.completed_tasks += 1;
        self.current_complexity_score = complexity.max(0.0);
        self.last_active_epoch_s = epoch_s;
        self.completion_history.push(duration_ms as f64);
        if self.completion_history.len() > HISTORY_CAP {
            self.completion_history.remove(0);
        }
        self.avg_completion_ms = self.avg_completion_mean() as f32;
    }

    /// `sum(history) / len(history)` in f64, in append order -- bit-comparable to the source's own
    /// sequential sum. Returns 0.0 for an empty history, as the source's default does.
    pub fn avg_completion_mean(&self) -> f64 {
        if self.completion_history.is_empty() {
            return 0.0;
        }
        let mut sum = 0.0f64;
        for v in &self.completion_history {
            sum += *v;
        }
        sum / self.completion_history.len() as f64
    }
}

// --- CAPABILITY ROUTER HELPER ---
/// The source's `_capabilities` DICT: role -> capability list, in dict order. `get_capable_roles` and
/// `redistribute_task`'s stable sort both depend on that order, and `get_swarm_load_summary` totals over
/// exactly these 12 roles -- so this table, not a council roster, is the balancer's oracle.
pub static CAPABILITIES: [(&str, &[&str]); 12] = [
    ("Logic", &["reasoning", "analysis", "math", "logic"]),
    ("Archi", &["architecture", "design", "planning", "review"]),
    ("Devo", &["implementation", "code", "development", "debugging"]),
    ("Shield", &["security", "audit", "validation", "compliance"]),
    ("Seeker", &["research", "discovery", "investigation", "intel"]),
    ("Bridge", &["integration", "api", "synthesis", "connection"]),
    ("Flow", &["pipeline", "automation", "devops", "deployment"]),
    ("Scribe", &["documentation", "writing", "reporting"]),
    ("Verify", &["testing", "qa", "verification", "assertion"]),
    ("Pulse", &["monitoring", "health", "telemetry", "alerts"]),
    ("Vision", &["ui", "design", "ux", "frontend"]),
    ("Orchestra", &["management", "coordination", "oversight"]),
];

/// The source's `capability in caps`: an unknown role holds NOTHING (the source's membership test has no
/// catch-all), so an unknown capability resolves to no roles rather than to every role.
pub fn role_matches_capability(role: &str, cap: &str) -> bool {
    match CAPABILITIES.iter().find(|(r, _)| *r == role) {
        Some((_, caps)) => caps.contains(&cap),
        None => false,
    }
}

// --- COGNITIVE LOAD BALANCER ---
pub struct CognitiveLoadBalancer {
    /// Public so a caller (and the routes) can read the roster without a second copy of it.
    pub agent_loads: Vec<AgentLoadMetrics>,
    pub total_routings: u64,
    /// Size of the roster the balancer was built with. The source's `get_swarm_load_summary` iterates
    /// `self._capabilities` (its roster), so a role created later by `record_task_start` is visible to
    /// `get_agent_load` but absent from the summary -- reproduced rather than smoothed over.
    pub roster_len: usize,
}

impl CognitiveLoadBalancer {
    /// The source's `__init__`: an EMPTY `_agent_load` plus the 12-role `_capabilities` roster, and no
    /// clock. The source reads `time.time()` inside its recording methods; this port takes that stamp as
    /// an argument instead, so the identical call sequence is replayable here and against the oracle.
    pub fn new() -> Self {
        let mut loads = Vec::with_capacity(CAPABILITIES.len());
        for (role, _caps) in CAPABILITIES.iter() {
            loads.push(AgentLoadMetrics::new(role));
        }

        Self {
            agent_loads: loads,
            total_routings: 0,
            roster_len: CAPABILITIES.len(),
        }
    }

    pub fn select_best_agent(&mut self, required_capability: &str, task_complexity: f32, epoch_s: u64) -> String {
        self.total_routings += 1;

        // Find candidate agents capable of handling this domain
        let mut candidates: Vec<usize> = self.agent_loads.iter().enumerate()
            .filter(|(_, a)| role_matches_capability(&a.role, required_capability))
            .map(|(i, _)| i)
            .collect();

        // If no strict match, fallback to all agents
        if candidates.is_empty() {
            candidates = (0..self.agent_loads.len()).collect();
        }

        // Pick candidate with minimum cognitive load score
        let mut best_idx = candidates[0];
        let mut min_load = self.agent_loads[best_idx].cognitive_load();

        for &idx in &candidates[1..] {
            let load = self.agent_loads[idx].cognitive_load();
            if load < min_load {
                min_load = load;
                best_idx = idx;
            }
        }

        self.agent_loads[best_idx].assign_task(task_complexity, epoch_s);
        self.agent_loads[best_idx].role.clone()
    }

    pub fn record_task_completion(&mut self, role: &str, duration_ms: f32, complexity: f32, epoch_s: u64) {
        if let Some(agent) = self.agent_loads.iter_mut().find(|a| a.role == role) {
            agent.complete_task(duration_ms, complexity, epoch_s);
        }
    }

    pub fn get_agent_metrics(&self, role: &str) -> Option<&AgentLoadMetrics> {
        self.agent_loads.iter().find(|a| a.role == role)
    }

    pub fn total_active_swarm_tasks(&self) -> u32 {
        self.agent_loads.iter().map(|a| a.active_tasks).sum()
    }

    // --- The source's five balancer entry points, ported rule-for-rule ---

    /// Source's `record_task_start(agent_role)`: creates the role if unknown, pending count up,
    /// last-active stamped. Complexity is NOT touched here (the source sets it at completion).
    pub fn record_task_start(&mut self, role: &str, epoch_s: u64) -> bool {
        if !self.agent_loads.iter().any(|a| a.role == role) {
            self.agent_loads.push(AgentLoadMetrics::new(role));
        }
        if let Some(agent) = self.agent_loads.iter_mut().find(|a| a.role == role) {
            agent.assign_task(0.0, epoch_s);
            return true;
        }
        false
    }

    /// Source's `record_task_end(agent_role, completion_time, complexity)`. An unknown role is a
    /// no-op returning false, exactly as the source returns early.
    pub fn record_task_end(&mut self, role: &str, completion_ms: f32, complexity: f32, epoch_s: u64) -> bool {
        match self.agent_loads.iter_mut().find(|a| a.role == role) {
            Some(agent) => {
                agent.complete_task(completion_ms, complexity, epoch_s);
                true
            }
            None => false,
        }
    }

    /// Source's `get_agent_load(agent_role)`: declared defaults for an unknown role.
    pub fn get_agent_load(&self, role: &str) -> AgentLoadView {
        match self.agent_loads.iter().find(|a| a.role == role) {
            Some(a) => AgentLoadView {
                role: a.role.clone(),
                task_count: a.active_tasks,
                avg_completion_time: a.avg_completion_mean(),
                current_complexity: a.current_complexity_score as f64,
                last_active: a.last_active_epoch_s as f64,
                load_score: a.active_tasks as f64 + a.current_complexity_score as f64 * 0.5,
            },
            None => AgentLoadView {
                role: String::from(role),
                task_count: 0,
                avg_completion_time: 0.0,
                current_complexity: 0.0,
                last_active: 0.0,
                load_score: 0.0,
            },
        }
    }

    /// Source's `get_swarm_load_summary()`: the declared roster only, totals over it, and
    /// `round(active/total*100, 1)`. The tenth is computed on exact integer arithmetic with Python's
    /// half-to-even tie rule, not by scaling a float (which would break the ties the source rounds).
    pub fn get_swarm_load_summary(&self) -> SwarmLoadSummary {
        let roster = &self.agent_loads[..self.roster_len.min(self.agent_loads.len())];
        let agents: Vec<AgentLoadView> = roster.iter().map(|a| self.get_agent_load(&a.role)).collect();
        let total_pending_tasks: u64 = agents.iter().map(|a| a.task_count as u64).sum();
        let active_agents: u64 = agents.iter().filter(|a| a.task_count > 0).count() as u64;
        let total_agents = agents.len() as u64;

        let utilization_tenths = if total_agents == 0 {
            0u64
        } else {
            let n = active_agents * 1000;
            let q = n / total_agents;
            let r = n % total_agents;
            if r * 2 > total_agents || (r * 2 == total_agents && q % 2 == 1) {
                q + 1
            } else {
                q
            }
        };

        SwarmLoadSummary {
            agents,
            total_pending_tasks,
            active_agents,
            total_agents,
            utilization_pct: utilization_tenths as f64 / 10.0,
        }
    }

    /// Source's `get_capable_roles(capability)`: every roster role holding that capability, in roster order.
    pub fn get_capable_roles(&self, capability: &str) -> Vec<String> {
        self.agent_loads[..self.roster_len.min(self.agent_loads.len())]
            .iter()
            .filter(|a| role_matches_capability(&a.role, capability))
            .map(|a| a.role.clone())
            .collect()
    }

    /// Source's `redistribute_task(task_description, preferred_roles)`: score is
    /// `task_count + current_complexity * 0.5`, lowest wins, ties resolve to the earlier entry in
    /// `preferred_roles` (the source's stable sort), unknown roles are skipped, an empty preference
    /// list means the whole roster, and NO candidates means None. It selects only -- it does not
    /// record a start, because the source does not.
    pub fn redistribute_task(&self, _task_description: &str, preferred: &[String]) -> Option<String> {
        let roster = &self.agent_loads[..self.roster_len.min(self.agent_loads.len())];
        // The source skips any preferred role absent from `_capabilities`, so a role created later by
        // `record_task_start` is not a redistribution candidate even when it is named explicitly.
        let candidates: Vec<&AgentLoadMetrics> = if preferred.is_empty() {
            roster.iter().collect()
        } else {
            preferred
                .iter()
                .filter_map(|role| roster.iter().find(|a| &a.role == role))
                .collect()
        };
        if candidates.is_empty() {
            return None;
        }

        let mut best_role = candidates[0].role.clone();
        let mut best_score = candidates[0].active_tasks as f64
            + candidates[0].current_complexity_score as f64 * 0.5;
        for c in &candidates[1..] {
            let score = c.active_tasks as f64 + c.current_complexity_score as f64 * 0.5;
            if score < best_score {
                best_score = score;
                best_role = c.role.clone();
            }
        }
        Some(best_role)
    }
}

/// The source's `get_agent_load` dict.
#[derive(Debug, Clone)]
pub struct AgentLoadView {
    pub role: String,
    pub task_count: u32,
    pub avg_completion_time: f64,
    pub current_complexity: f64,
    pub last_active: f64,
    pub load_score: f64,
}

/// The source's `get_swarm_load_summary` dict (its `timestamp` field is the caller's clock, so it is
/// not produced here -- the module has no clock and will not invent one).
#[derive(Debug, Clone)]
pub struct SwarmLoadSummary {
    pub agents: Vec<AgentLoadView>,
    pub total_pending_tasks: u64,
    pub active_agents: u64,
    pub total_agents: u64,
    pub utilization_pct: f64,
}

// --- SINGLETON INTERFACE ---
static SINGLETON_LOAD_BALANCER: AtomicU64 = AtomicU64::new(0);

pub fn get_load_balancer() -> u64 {
    let v = SINGLETON_LOAD_BALANCER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_LOAD_BALANCER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

