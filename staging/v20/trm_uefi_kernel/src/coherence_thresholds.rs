//! TRM Swarm OS — Coherence Threshold Framework (real no_std port).
//! Ported from swarm_v2/core/coherence_thresholds.py (Phase 4 Shell-01
//! Recursive Self-Healing): system-stability class constants plus the
//! metric-based `check_stability` verdict logic.
//!
//! NOTE: the previous stub exposed a fabricated coherence-band API
//! (COHERENCE_MIN 0.45 / COHERENCE_MAX 0.95 / STABILITY_WINDOW 8) that does
//! NOT exist in the python source. It is replaced here by the real constants
//! and evaluation order from coherence_thresholds.py (cpu_load → memory_usage
//! → error_rate, matching python's fixed `if` chain).

extern crate alloc;

// ── System stability thresholds (python class attributes, percentages) ──────
pub const CPU_LOAD_WARNING: f64 = 80.0;
pub const MEMORY_USAGE_WARNING: f64 = 85.0;
pub const DISK_SPACE_WARNING: f64 = 90.0;

// ── Agent health thresholds ──────────────────────────────────────────────────
pub const MAX_RESPONSE_LATENCY: f64 = 5.0; // Seconds
pub const HEARTBEAT_TIMEOUT: u64 = 300; // Seconds (5 minutes)
pub const ERROR_RATE_LIMIT: f64 = 0.1; // 10% failure rate over a window

// ── Mesh integrity thresholds ────────────────────────────────────────────────
pub const MIN_ACTIVE_AGENTS: u32 = 3; // Minimum agents for a functional mesh
pub const MAX_ORPHANED_TASKS: u32 = 5; // Tasks routed but not picked up

// ── Remediation action levels (python LEVEL_1_WARNING … LEVEL_4_EMERGENCY) ──
pub const LEVEL_1_WARNING: &str = "LOG_WARNING";
pub const LEVEL_2_RESTART: &str = "RESTART_SERVICE";
pub const LEVEL_3_ROLLBACK: &str = "ROLLBACK_DEPLOYMENT";
pub const LEVEL_4_EMERGENCY: &str = "FULL_SYSTEM_RESET";
/// Verdict returned by `check_stability` when every metric is within range.
pub const STABLE: &str = "STABLE";

/// Stability verdict — the finite set of strings python's
/// `check_stability(metrics)` returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StabilityVerdict {
    Stable,
    Level1Warning,
    Level2Restart,
}

impl StabilityVerdict {
    /// String form, 1:1 with the python return values.
    pub fn as_str(self) -> &'static str {
        match self {
            StabilityVerdict::Stable => STABLE,
            StabilityVerdict::Level1Warning => LEVEL_1_WARNING,
            StabilityVerdict::Level2Restart => LEVEL_2_RESTART,
        }
    }
}

/// Numeric metrics snapshot — replaces python's `metrics: Dict[str, float]`.
/// Fields mirror the dict keys `check_stability` reads; python defaults each
/// absent key to 0.
#[derive(Clone, Copy, Debug, Default)]
pub struct StabilityMetrics {
    pub cpu_load: f64,
    pub memory_usage: f64,
    pub error_rate: f64,
}

/// python `CoherenceThresholds.check_stability(metrics)` — fixed evaluation
/// order: cpu_load > 80 → LEVEL_1; memory_usage > 85 → LEVEL_1;
/// error_rate > 0.1 → LEVEL_2; otherwise STABLE. Returns the verdict enum;
/// use `verdict.as_str()` for the python string.
pub fn check_stability(metrics: &StabilityMetrics) -> StabilityVerdict {
    if metrics.cpu_load > CPU_LOAD_WARNING {
        StabilityVerdict::Level1Warning
    } else if metrics.memory_usage > MEMORY_USAGE_WARNING {
        StabilityVerdict::Level1Warning
    } else if metrics.error_rate > ERROR_RATE_LIMIT {
        StabilityVerdict::Level2Restart
    } else {
        StabilityVerdict::Stable
    }
}

/// String-returning variant for callers that mirror python's API directly.
pub fn check_stability_str(cpu_load: f64, memory_usage: f64, error_rate: f64) -> &'static str {
    check_stability(&StabilityMetrics {
        cpu_load,
        memory_usage,
        error_rate,
    })
    .as_str()
}

/// Threshold snapshot — python `CoherenceThresholds.get_thresholds()`, which
/// returns every non-underscore, non-callable class attribute as a dict
/// (numeric thresholds AND the LEVEL_* strings).
#[derive(Clone, Copy, Debug)]
pub struct ThresholdSet {
    pub cpu_load_warning: f64,
    pub memory_usage_warning: f64,
    pub disk_space_warning: f64,
    pub max_response_latency: f64,
    pub heartbeat_timeout: u64,
    pub error_rate_limit: f64,
    pub min_active_agents: u32,
    pub max_orphaned_tasks: u32,
    pub level_1_warning: &'static str,
    pub level_2_restart: &'static str,
    pub level_3_rollback: &'static str,
    pub level_4_emergency: &'static str,
}

/// All thresholds — 1:1 values from the python class body.
pub fn get_thresholds() -> ThresholdSet {
    ThresholdSet {
        cpu_load_warning: CPU_LOAD_WARNING,
        memory_usage_warning: MEMORY_USAGE_WARNING,
        disk_space_warning: DISK_SPACE_WARNING,
        max_response_latency: MAX_RESPONSE_LATENCY,
        heartbeat_timeout: HEARTBEAT_TIMEOUT,
        error_rate_limit: ERROR_RATE_LIMIT,
        min_active_agents: MIN_ACTIVE_AGENTS,
        max_orphaned_tasks: MAX_ORPHANED_TASKS,
        level_1_warning: LEVEL_1_WARNING,
        level_2_restart: LEVEL_2_RESTART,
        level_3_rollback: LEVEL_3_ROLLBACK,
        level_4_emergency: LEVEL_4_EMERGENCY,
    }
}
