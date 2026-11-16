//! TRM Swarm OS — cognitive_load_balancer (no_std port, REAL-verified).
//! 1:1 deterministic core of swarm_v2/core/cognitive_load_balancer.py:
//!   - CAPABILITY_TABLE: 12 roles x capability keywords (python dict order)
//!   - record_task_start / record_task_end (moving avg, max-50 history,
//!     complexity clamp >= 0, task_count floor 0)
//!   - get_agent_load (default-merged), get_swarm_load_summary
//!     (utilization_pct = python round(active/12*100, 1))
//!   - redistribute_task: score = task_count + current_complexity*0.5, stable
//!     sort -> least-loaded capable role (python dict order tie-break)
//!   - get_capable_roles
//! Wall-clock reads (time.time()) are injected as `now` parameters.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// Role -> capability keyword table, python dict insertion order (1:1).
pub const CAPABILITY_TABLE: &[(&str, &[&str])] = &[
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

/// Role order used when preferred_roles is empty (python capability keys).
pub const ROLE_ORDER: &[&str] = &[
    "Logic", "Archi", "Devo", "Shield", "Seeker", "Bridge", "Flow", "Scribe",
    "Verify", "Pulse", "Vision", "Orchestra",
];

/// Python _max_history = 50.
pub const MAX_HISTORY: usize = 50;

/// Per-agent load metrics mirroring the python _agent_load[role] dict
/// {task_count, avg_completion_time, current_complexity, last_active}.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadMetrics {
    pub task_count: i64,
    pub avg_completion_time: f64,
    pub current_complexity: f64,
    pub last_active: f64,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        LoadMetrics {
            task_count: 0,
            avg_completion_time: 0.0,
            current_complexity: 0.0,
            last_active: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct AgentEntry {
    metrics: LoadMetrics,
    history: VecDeque<f64>, // _completion_history[role], max 50
}

impl AgentEntry {
    fn new(now: f64) -> Self {
        AgentEntry {
            metrics: LoadMetrics { last_active: now, ..LoadMetrics::default() },
            history: VecDeque::new(),
        }
    }
}

/// Aggregate load distribution across the swarm (12 roles always reported,
/// python iterates _capabilities and merges defaults per role).
#[derive(Debug, Clone, PartialEq)]
pub struct SwarmSummary {
    pub roles: Vec<(String, LoadMetrics)>,
    pub total_pending_tasks: i64,
    pub active_agents: usize,
    pub total_agents: usize,
    pub utilization_pct: f64,
}

fn capability_has_role(role: &str) -> bool {
    CAPABILITY_TABLE.iter().any(|(r, _)| *r == role)
}

/// python round-half-even at 1 decimal for non-negative values (no_std).
fn round_1dec(x: f64) -> f64 {
    let q = x * 10.0;
    let t = q as i64; // trunc toward zero; callers pass x >= 0
    let f = q - t as f64;
    let mut r = if f < 0.5 {
        t
    } else if f > 0.5 {
        t + 1
    } else if t % 2 == 0 {
        t
    } else {
        t + 1
    };
    r as f64 / 10.0
}

/// Tracks per-agent load and redistributes tasks to the least-loaded capable
/// agent.
#[derive(Debug, Clone, Default)]
pub struct CognitiveLoadBalancer {
    agents: BTreeMap<String, AgentEntry>,
}

impl CognitiveLoadBalancer {
    /// python `__init__` (capability table is a module const here).
    pub fn new() -> Self {
        CognitiveLoadBalancer { agents: BTreeMap::new() }
    }

    /// python `record_task_start(agent_role)`: creates the entry on first
    /// sight (task_count 0, complexity 0, last_active=now), then increments
    /// task_count and stamps last_active.
    pub fn record_task_start(&mut self, agent_role: &str, now: f64) {
        let e = self
            .agents
            .entry(agent_role.to_string())
            .or_insert_with(|| AgentEntry::new(now));
        e.metrics.task_count += 1;
        e.metrics.last_active = now;
    }

    /// python `record_task_end(agent_role, completion_time, complexity)`:
    /// no-op (returns false) for unknown roles. Sets current_complexity =
    /// max(0, complexity), decrements task_count to a floor of 0, appends
    /// completion_time to the history (trimming to newest 50) and recomputes
    /// avg_completion_time = mean(history).
    pub fn record_task_end(
        &mut self,
        agent_role: &str,
        completion_time: f64,
        complexity: f64,
        now: f64,
    ) -> bool {
        let entry = match self.agents.get_mut(agent_role) {
            Some(e) => e,
            None => return false,
        };
        entry.metrics.current_complexity = if complexity < 0.0 { 0.0 } else { complexity };
        entry.metrics.last_active = now;
        entry.metrics.task_count = if entry.metrics.task_count > 0 {
            entry.metrics.task_count - 1
        } else {
            0
        };
        entry.history.push_back(completion_time);
        while entry.history.len() > MAX_HISTORY {
            entry.history.pop_front();
        }
        let sum: f64 = entry.history.iter().fold(0.0, |a, &x| a + x);
        entry.metrics.avg_completion_time = sum / entry.history.len() as f64;
        true
    }

    /// python `get_agent_load(agent_role)`: defaults merged with stored
    /// metrics — unknown roles report the zero default.
    pub fn get_agent_load(&self, agent_role: &str) -> LoadMetrics {
        match self.agents.get(agent_role) {
            Some(e) => e.metrics,
            None => LoadMetrics::default(),
        }
    }

    /// python `get_swarm_load_summary()`: reports all 12 capability roles,
    /// total pending tasks, active agents and utilization_pct =
    /// round(active_agents / 12 * 100, 1).
    pub fn get_swarm_load_summary(&self) -> SwarmSummary {
        let mut roles: Vec<(String, LoadMetrics)> = Vec::with_capacity(CAPABILITY_TABLE.len());
        let mut total_pending: i64 = 0;
        let mut active: usize = 0;
        for (role, _) in CAPABILITY_TABLE {
            let m = self.get_agent_load(role);
            if m.task_count > 0 {
                active += 1;
            }
            total_pending += m.task_count;
            roles.push((role.to_string(), m));
        }
        let total_agents = CAPABILITY_TABLE.len();
        let utilization_pct = if total_agents == 0 {
            0.0
        } else {
            round_1dec((active as f64 / total_agents as f64) * 100.0)
        };
        SwarmSummary {
            roles,
            total_pending_tasks: total_pending,
            active_agents: active,
            total_agents,
            utilization_pct,
        }
    }

    /// python `redistribute_task(task_description, preferred_roles)`: score =
    /// task_count + current_complexity * 0.5 for each preferred role present
    /// in the capability table; python sorts by score with a stable sort, so
    /// ties keep the preferred-order position. Returns the winner or None
    /// when no preferred role is a known capability. The task_description
    /// argument is unused in the python body too.
    pub fn redistribute_task(&self, _task_description: &str, preferred_roles: &[String]) -> Option<String> {
        let mut roles: Vec<&str> = preferred_roles.iter().map(|s| s.as_str()).collect();
        if roles.is_empty() {
            roles = ROLE_ORDER.to_vec();
        }
        let mut candidates: Vec<(f64, String)> = Vec::new();
        for role in roles {
            if !capability_has_role(role) {
                continue;
            }
            let m = self.get_agent_load(role);
            let score = m.task_count as f64 + m.current_complexity * 0.5;
            candidates.push((score, role.to_string()));
        }
        if candidates.is_empty() {
            return None;
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0)); // stable: ties keep order
        Some(candidates[0].1.clone())
    }

    /// python `get_capable_roles(capability)`: roles (table order) whose
    /// keyword list contains the capability.
    pub fn get_capable_roles(&self, capability: &str) -> Vec<String> {
        let mut out = Vec::new();
        for (role, caps) in CAPABILITY_TABLE {
            if caps.iter().any(|c| *c == capability) {
                out.push(role.to_string());
            }
        }
        out
    }
}

/// Python module-level singleton `get_load_balancer()` — no_std handle
/// convention: 1 once initialized.
static SINGLETON_LOAD_BALANCER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_load_balancer() -> u64 {
    let v = SINGLETON_LOAD_BALANCER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_LOAD_BALANCER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
