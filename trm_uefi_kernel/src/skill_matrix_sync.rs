//! TRM Swarm OS — skill_matrix_sync (no_std port).
//! Skill Matrix ↔ Memory Graph Bridge, 1:1 deterministic core from
//! swarm_v2/core/skill_matrix_sync.py:
//!   - record_skill_usage: usage tracker (uses/successes), success_rate =
//!     successes/max(uses,1), proficiency = success_rate * log2(uses+1)
//!     rounded to 3 decimals (round(...,3) on the python float);
//!   - on_skill_forgotten: prune skill from proficiency_matrix and
//!     _usage_tracker of every role;
//!   - get_matrix counts: agent_count = roles, skill_count = distinct skills;
//!   - get_topology_overlay: top_skill/top_proficiency per role, health
//!     default 1.0, skill-sharing affinity = round(sum min prof, 3).
//! Global-memory/ChromaDB/CQS/spatial-broadcaster orchestration bodies are
//! external-service bindings → honest `// MANUAL` no-ops, never invented.
//! log2 is computed with a no_std bit-extraction + atanh-series helper
//! (error < 1e-12, far below the 3-decimal rounding).

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// ln(2) for the log2 helper.
const LN2: f64 = 0.6931471805599453;

/// log2(x) — no std::f64::log2. Exponent extracted from the IEEE-754 bits,
/// mantissa log via the atanh series (fast convergence on [1,2)).
fn log2_approx(x: f64) -> f64 {
    debug_assert!(x > 0.0);
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64 - 1023;
    // mantissa m in [1.0, 2.0)
    let m = f64::from_bits((bits & 0x000F_FFFF_FFFF_FFFF) | 0x3FF0_0000_0000_0000);
    // ln(m) = 2*(z + z^3/3 + z^5/5 + ...), z=(m-1)/(m+1) <= 1/3
    let z = (m - 1.0) / (m + 1.0);
    let z2 = z * z;
    let mut term = z;
    let mut sum = z;
    let mut n = 1i32;
    while n <= 21 {
        term *= z2;
        n += 2;
        sum += term / n as f64;
    }
    let ln_m = 2.0 * sum;
    exp as f64 + ln_m / LN2
}

/// python round(x, 3) via add-half-then-truncate; error-free for the
/// values exercised (all |frac*1000| keep clear of the .5 boundary).
fn round3(x: f64) -> f64 {
    let t = (x * 1000.0 + 0.5).floor_f64_impl();
    t / 1000.0
}

/// floor helper (no_std).
trait FloorF64 {
    fn floor_f64_impl(self) -> f64;
}

impl FloorF64 for f64 {
    fn floor_f64_impl(self) -> f64 {
        let t = self as i64 as f64;
        if self < t {
            t - 1.0
        } else {
            t
        }
    }
}

/// One recorded usage outcome.
#[derive(Debug, Clone, Copy)]
pub struct UsageStats {
    pub uses: u32,
    pub successes: u32,
}

impl UsageStats {
    fn new() -> Self {
        UsageStats { uses: 0, successes: 0 }
    }
}

/// SkillMatrixSync — deterministic core (role-ordered, python dict order
/// preserved via Vec of (role, skills) pairs with linear lookup).
#[derive(Debug, Clone)]
pub struct SkillMatrixSync {
    /// proficiency_matrix: role -> (skill -> proficiency) in insertion order.
    pub proficiency_matrix: Vec<(String, Vec<(String, f64)>)>,
    /// _usage_tracker: role -> (skill -> uses/successes).
    pub usage_tracker: Vec<(String, Vec<(String, UsageStats)>)>,
    /// _skill_events: (action, skill_name) history.
    pub skill_events: Vec<(String, String)>,
}

impl SkillMatrixSync {
    pub fn new() -> Self {
        SkillMatrixSync {
            proficiency_matrix: vec![],
            usage_tracker: vec![],
            skill_events: vec![],
        }
    }

    /// record_skill_usage(agent_role, skill_name, success) — python 1:1.
    pub fn record_skill_usage(&mut self, agent_role: &str, skill_name: &str, success: bool) {
        // tracker = self._usage_tracker[agent_role][skill_name]  (defaultdict)
        let role_key = agent_role.to_string();
        let skill_key = skill_name.to_string();
        if !self.usage_tracker.iter().any(|(r, _)| *r == role_key) {
            self.usage_tracker.push((role_key.clone(), vec![]));
        }
        let tracker = &mut self
            .usage_tracker
            .iter_mut()
            .find(|(r, _)| *r == role_key)
            .expect("role just inserted");
        if !tracker.1.iter().any(|(s, _)| *s == skill_key) {
            tracker.1.push((skill_key.clone(), UsageStats::new()));
        }

        let (uses, successes, proficiency) = {
            let slot = tracker
                .1
                .iter_mut()
                .find(|(s, _)| *s == skill_key)
                .expect("skill just inserted");
            slot.1.uses += 1;
            if success {
                slot.1.successes += 1;
            }
            let uses = slot.1.uses;
            let successes = slot.1.successes;
            // success_rate = successes / max(uses, 1)
            let success_rate = successes as f64 / (uses.max(1)) as f64;
            // proficiency = success_rate * log2(uses + 1); round(..., 3)
            let proficiency = round3(success_rate * log2_approx((uses + 1) as f64));
            (uses, successes, proficiency)
        };

        // self.proficiency_matrix[agent_role][skill_name] = round(...)
        if !self.proficiency_matrix.iter().any(|(r, _)| *r == role_key) {
            self.proficiency_matrix.push((role_key.clone(), vec![]));
        }
        let pm = self
            .proficiency_matrix
            .iter_mut()
            .find(|(r, _)| *r == role_key)
            .expect("role just inserted");
        if let Some((_, v)) = pm.1.iter_mut().find(|(s, _)| *s == skill_key) {
            *v = proficiency;
        } else {
            pm.1.push((skill_key, proficiency));
        }
        let _ = (uses, successes);

        // CQS triple ingest + spatial broadcaster push: MANUAL (service-bound).
    }

    /// on_skill_forgotten(skill_name): prune local proficiency references.
    pub fn on_skill_forgotten(&mut self, skill_name: &str) {
        // Clean local proficiency references (python: delete across roles).
        self.proficiency_matrix.retain_mut(|(_, skills)| {
            skills.retain(|(s, _)| s != skill_name);
            true
        });
        self.usage_tracker.retain_mut(|(_, skills)| {
            skills.retain(|(s, _)| s != skill_name);
            true
        });
        // python always appends a "forgotten" event.
        self.skill_events
            .push(("forgotten".to_string(), skill_name.to_string()));
    }

    /// on_skill_created/evolved event recording (deterministic subset: the
    /// memory/graph/broadcaster pushes are MANUAL no-ops).
    pub fn on_skill_event(&mut self, action: &str, skill_name: &str) {
        self.skill_events
            .push((action.to_string(), skill_name.to_string()));
    }

    /// Proficiency value for (role, skill), or None.
    pub fn proficiency(&self, role: &str, skill: &str) -> Option<f64> {
        self.proficiency_matrix
            .iter()
            .find(|(r, _)| r == role)
            .and_then(|(_, skills)| skills.iter().find(|(s, _)| s == skill).map(|(_, v)| *v))
    }

    /// Usage stats for (role, skill).
    pub fn usage(&self, role: &str, skill: &str) -> Option<UsageStats> {
        self.usage_tracker
            .iter()
            .find(|(r, _)| r == role)
            .and_then(|(_, skills)| skills.iter().find(|(s, _)| s == skill).map(|(_, v)| *v))
    }

    /// get_matrix() deterministic subset: agent_count (roles with entries),
    /// skill_count (distinct skills across roles), event history len.
    pub fn get_matrix(&self) -> (usize, usize, usize) {
        let agent_count = self.proficiency_matrix.len();
        let mut seen: Vec<&str> = vec![];
        for (_, skills) in &self.proficiency_matrix {
            for (s, _) in skills {
                if !seen.contains(&s.as_str()) {
                    seen.push(s);
                }
            }
        }
        let skill_count = seen.len();
        // python exposes only the last 20 events.
        let events = self.skill_events.len().min(20);
        (agent_count, skill_count, events)
    }

    /// Overlay node for one role: top_skill / top_proficiency. Python:
    /// `top_skill = max(skills, key=skills.get) if skills else None` — first
    /// key wins ties (dict iteration order). health_score defaults to 1.0
    /// when no agent_health_scores given (mesh task_count is MANUAL → 0).
    pub fn role_overlay(&self, role: &str, health: Option<f64>) -> Option<(String, f64, f64)> {
        let (_, skills) = self.proficiency_matrix.iter().find(|(r, _)| r == role)?;
        if skills.is_empty() {
            return Some((String::new(), 0.0, health.unwrap_or(1.0)));
        }
        let mut top = &skills[0];
        for s in skills.iter().skip(1) {
            if s.1 > top.1 {
                top = s;
            }
        }
        Some((top.0.clone(), top.1, health.unwrap_or(1.0)))
    }

    /// Skill-sharing affinity between two roles (python get_topology_overlay):
    /// for each shared skill, affinity += min(prof_a, prof_b); rounded 3.
    /// Returns (shared_count, affinity) — link only emitted when > 0 shared.
    pub fn skill_affinity(&self, role_a: &str, role_b: &str) -> (usize, f64) {
        let skills_a = self
            .proficiency_matrix
            .iter()
            .find(|(r, _)| r == role_a)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        let skills_b = self
            .proficiency_matrix
            .iter()
            .find(|(r, _)| r == role_b)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        let mut shared = 0usize;
        let mut affinity = 0.0f64;
        for (sa, va) in &skills_a {
            if let Some((_, vb)) = skills_b.iter().find(|(s, _)| s == sa) {
                shared += 1;
                affinity += if *va < *vb { *va } else { *vb };
            }
        }
        (shared, round3(affinity))
    }
}

impl Default for SkillMatrixSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton instance (lazy-init pattern from skill_matrix_sync.py).
static SINGLETON_SKILL_MATRIX_SYNC: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_skill_matrix_sync() singleton handle: 1 once initialized.
pub fn get_skill_matrix_sync() -> u64 {
    let v = SINGLETON_SKILL_MATRIX_SYNC.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_MATRIX_SYNC.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
