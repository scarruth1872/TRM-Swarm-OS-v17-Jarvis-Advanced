//! TRM Swarm OS — daily_evolution_engine (no_std port).
//! 1:1 with swarm_v2/core/daily_evolution_engine.py: the 14-domain
//! curriculum (topics per domain) + K3 determinism-based distribution.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Evolution step counter.
static EVOLUTION_STEP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// The 14 evolution domains (from python generate_curriculum).
pub const EVOLUTION_DOMAINS: &'static [&'static str] = &[
    "DESIGN", "PROTECT", "PROVE", "BUILD", "DISCOVER", "INTERFACE", "HEAL",
    "UNDERSTAND", "RECORD", "COORDINATE", "OBSERVE", "VERIFY", "CONNECT", "EVOLVE",
];

/// Curriculum topic pool per domain — 1:1 from python `curriculum_topics` (full pools;
/// python draws `random.sample(topics, min(5, len(topics)))`, so pool *size* matters:
/// DESIGN/DISCOVER/UNDERSTAND have 7 topics, the other 11 have 5).
pub const CURRICULUM: &'static [(&'static str, &'static [&'static str])] = &[
    ("DESIGN", &["VLSI layout", "molecular docking", "habitat engineering", "nanobot design", "quantum circuit layout", "creative topological metaphor synthesis", "sacred geometry narratives"]),
    ("PROTECT", &["zero-day detection", "APT pattern recognition", "DDR antibody evolution", "supply chain audit", "post-quantum crypto"]),
    ("PROVE", &["automated theorem proving", "SAT solver optimization", "dependent type theory", "homotopy type theory", "formal verification of LLMs"]),
    ("BUILD", &["Rust async runtime", "Zig cross-compilation", "WebGPU compute", "eBPF kernel modules", "WASM component model"]),
    ("DISCOVER", &["MCTS hypothesis tree search", "multi-source literature triangulation", "exoplanet atmospheres", "protein folding prediction", "dark matter detection", "consciousness correlates", "historical text analysis"]),
    ("INTERFACE", &["WCAG AAA compliance", "haptic feedback design", "voice synthesis", "AR spatial computing", "brain-computer interface"]),
    ("HEAL", &["immunotherapy optimization", "senolytic screening", "microbiome modulation", "gene therapy delivery", "personalized dosing"]),
    ("UNDERSTAND", &["affective resonance modeling", "cognitive load adaptation", "cross-cultural empathy", "patient narrative analysis", "emotional intelligence", "trauma-informed care", "phenomenological mapping"]),
    ("RECORD", &["clinical trial design", "FDA submission automation", "adverse event detection", "real-world evidence", "regulatory harmonization"]),
    ("COORDINATE", &["multi-agent consensus", "fault-tolerant scheduling", "load-balancing proofs", "priority queuing theory", "distributed ledger"]),
    ("OBSERVE", &["CMB anisotropy mapping", "gravitational wave detection", "neutrino astronomy", "spectral line analysis", "photon entanglement"]),
    ("VERIFY", &["mutation testing", "fuzzing strategies", "invariant generation", "property-based testing", "concolic execution"]),
    ("CONNECT", &["drug-target binding", "protein-protein interaction", "metabolic pathway mapping", "neural connectome", "ecological network"]),
    ("EVOLVE", &["meta-learning", "self-modifying code", "recursive self-improvement", "gradient descent on architecture", "emergent behavior"]),
];

/// Curriculum spans the 14 evolution domains.
pub fn generate_curriculum() -> u64 {
    CURRICULUM.len() as u64
}

/// K3 distribution by domain determinism (from python run_daily_evolution post-CP-483BAE85):
/// PROVE/EVOLVE: 0.94/0.04/0.02 | DESIGN/BUILD/PROTECT: 0.82/0.10/0.08
/// UNDERSTAND: 0.85/0.10/0.05 | HEAL/CONNECT: 0.72/0.20/0.08
/// DISCOVER: 0.76/0.18/0.06 | OBSERVE: 0.60/0.30/0.10
pub fn evolve() -> u64 {
    EVOLUTION_STEP.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let step = EVOLUTION_STEP.load(core::sync::atomic::Ordering::Relaxed);
    match step % 6 {
        0 => 94, // PROVE/EVOLVE -> 94% TRUE
        1 => 82, // DESIGN/BUILD/PROTECT -> 82% TRUE
        2 => 85, // UNDERSTAND (Flow Affective Resonance) -> 85% TRUE
        3 => 76, // DISCOVER (Seeker MCTS Triangulation) -> 76% TRUE
        4 => 72, // HEAL/CONNECT -> 72% TRUE
        _ => 60, // OBSERVE -> 60% TRUE
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// PORTED RULES — python `run_daily_evolution()` / `simulate_daily_evolution_native()`
// ══════════════════════════════════════════════════════════════════════════════

/// python `TASKS_PER_DAY = 1_000_000_000_000`
pub const TASKS_PER_DAY: u64 = 1_000_000_000_000;
/// python `AGENTS = 584`
pub const AGENTS: u64 = 584;
/// python: `steps = tasks * 64000`
pub const STEPS_PER_TASK: u64 = 64_000;
/// python curriculum: `min(5, len(topics))` topics/day at 100_000 tasks each -> 500_000.
pub const CURRICULUM_TOPICS_PER_DAY: usize = 5;
pub const TASKS_PER_TOPIC: u64 = 100_000;
pub const CURRICULUM_TASKS_PER_DAY: u64 = 500_000;

/// One row of the python `DOMAINS` dict (insertion order == iteration order).
pub struct DomainInfo {
    pub agent: &'static str,
    pub domain: &'static str,
    pub pct: f64,
    pub datasets: u32,
}

/// python `DOMAINS` — 14 agent domains, dict order preserved.
pub const DOMAINS: &'static [DomainInfo] = &[
    DomainInfo { agent: "Archi", domain: "DESIGN", pct: 0.12, datasets: 130 },
    DomainInfo { agent: "Shield", domain: "PROTECT", pct: 0.12, datasets: 18 },
    DomainInfo { agent: "Logic", domain: "PROVE", pct: 0.10, datasets: 106 },
    DomainInfo { agent: "Devo", domain: "BUILD", pct: 0.12, datasets: 176 },
    DomainInfo { agent: "Seeker", domain: "DISCOVER", pct: 0.10, datasets: 150 },
    DomainInfo { agent: "Jarvis", domain: "INTERFACE", pct: 0.06, datasets: 88 },
    DomainInfo { agent: "Pulse", domain: "HEAL", pct: 0.08, datasets: 12 },
    DomainInfo { agent: "Flow", domain: "UNDERSTAND", pct: 0.06, datasets: 64 },
    DomainInfo { agent: "EventLedger", domain: "RECORD", pct: 0.04, datasets: 12 },
    DomainInfo { agent: "Orchestra", domain: "COORDINATE", pct: 0.04, datasets: 12 },
    DomainInfo { agent: "THzGateway", domain: "OBSERVE", pct: 0.06, datasets: 12 },
    DomainInfo { agent: "Verify", domain: "VERIFY", pct: 0.04, datasets: 12 },
    DomainInfo { agent: "Bridge", domain: "CONNECT", pct: 0.04, datasets: 12 },
    DomainInfo { agent: "Hermes", domain: "EVOLVE", pct: 0.02, datasets: 20 },
];

/// The (t,u,f) K3 chain of python `run_daily_evolution`, evaluated in elif order.
pub fn k3_ratios(domain: &str) -> (f64, f64, f64) {
    match domain {
        "PROVE" | "EVOLVE" => (0.94, 0.04, 0.02),
        "DESIGN" | "BUILD" | "PROTECT" => (0.82, 0.10, 0.08),
        "UNDERSTAND" => (0.85, 0.10, 0.05),
        "HEAL" | "CONNECT" => (0.72, 0.20, 0.08),
        "DISCOVER" => (0.76, 0.18, 0.06),
        "OBSERVE" => (0.60, 0.30, 0.10),
        _ => (0.78, 0.14, 0.08),
    }
}

/// python `round(x, 1)` — round-half-to-even on the scaled decimal, core-only
/// (no `f64::floor`/`round`: std-only float intrinsics, unavailable in no_std).
pub fn round_1(x: f64) -> f64 {
    let neg = x < 0.0;
    let a = if neg { -x } else { x };
    let scaled = a * 10.0;
    let t = scaled as i64;
    let frac = scaled - (t as f64);
    let mut r = t;
    if frac > 0.5 {
        r = t + 1;
    } else if frac == 0.5 && (t % 2) != 0 {
        r = t + 1;
    }
    let out = (r as f64) / 10.0;
    if neg { -out } else { out }
}

/// python `round(x)` -> int (ties to even), core-only.
pub fn round_i64(x: f64) -> i64 {
    let neg = x < 0.0;
    let a = if neg { -x } else { x };
    let t = a as i64;
    let frac = a - (t as f64);
    let mut r = t;
    if frac > 0.5 {
        r = t + 1;
    } else if frac == 0.5 && (t % 2) != 0 {
        r = t + 1;
    }
    if neg { -r } else { r }
}

/// Result of `simulate_daily_evolution_native`. The native strip (ctypes + host DLL)
/// is host-side; a no_std kernel always takes python's non-native branch, so `native`
/// is false and `total_true/unknown/false` stay 0.0 (python's fallback dict has no
/// such keys). `simulated_true` is python's `round(epochs * sum(pct) * 0.94, 1)`.
pub struct DailyEvolutionSim {
    pub epochs: i64,
    pub simulated_true: f64,
    pub total_true: f64,
    pub total_unknown: f64,
    pub total_false: f64,
    pub native: bool,
}

/// python `simulate_daily_evolution_native(epochs=1000)`, non-native branch 1:1.
pub fn simulate_daily_evolution_native(epochs: i64) -> DailyEvolutionSim {
    let mut sum_pct = 0.0f64;
    let mut i = 0usize;
    while i < DOMAINS.len() {
        sum_pct += DOMAINS[i].pct;
        i += 1;
    }
    let mut sim = DailyEvolutionSim {
        epochs,
        simulated_true: 0.0,
        total_true: 0.0,
        total_unknown: 0.0,
        total_false: 0.0,
        native: false,
    };
    // python: `if not _EVOLUTION_DLL or epochs <= 0:` -> fallback path.
    // On a stripped kernel the DLL branch cannot exist, so this is the live branch.
    let tot_true = ((epochs as f64) * sum_pct) * 0.94;
    sim.simulated_true = round_1(tot_true);
    sim
}

/// Per-agent row of the python `report['agent_results']` (+ the intermediate
/// `results[agent]` row it is derived from).
pub struct AgentResult {
    pub agent: &'static str,
    pub domain: &'static str,
    pub tasks: u64,
    pub k3_true: u64,
    pub k3_unknown: u64,
    pub k3_false: u64,
    pub training_steps: u64,
    pub k3_true_pct: f64,
}

/// python `report['k3']`.
pub struct K3Report {
    pub true_count: u64,
    pub true_pct: f64,
    pub unknown_count: u64,
    pub unknown_pct: f64,
    pub false_count: u64,
    pub false_pct: f64,
}

/// python `report` (minus the host-only `date` string and the random curriculum sample).
pub struct DailyEvolutionReport {
    pub day: u64,
    pub total_tasks: u64,
    pub agents: u64,
    pub domains: usize,
    pub k3: K3Report,
    pub training_steps: u64,
    pub execution_time_us: u64,
    pub agent_results: Vec<AgentResult>,
}

/// python `run_daily_evolution()` — the portable computation, 1:1.
/// `tasks = int(TASKS_PER_DAY * pct)`, `tc = int(tasks*t)`, `uc = int(tasks*u)`,
/// `fc = tasks - tc - uc`, `steps = tasks * 64000`, then K3 totals + percentages.
pub fn run_daily_evolution() -> DailyEvolutionReport {
    let mut agent_results: Vec<AgentResult> = Vec::with_capacity(DOMAINS.len());
    let mut total_true: u64 = 0;
    let mut total_unknown: u64 = 0;
    let mut total_false: u64 = 0;
    let mut total_steps: u64 = 0;

    let mut i = 0usize;
    while i < DOMAINS.len() {
        let d = &DOMAINS[i];
        let tasks = ((TASKS_PER_DAY as f64) * d.pct) as u64; // python int()
        let (t, u, _f) = k3_ratios(d.domain);
        let tc = ((tasks as f64) * t) as u64; // python int()
        let uc = ((tasks as f64) * u) as u64; // python int()
        let fc = tasks - tc - uc; // python: tasks - tc - uc
        let steps = tasks * STEPS_PER_TASK;

        total_steps += steps;
        total_true += tc;
        total_unknown += uc;
        total_false += fc;

        agent_results.push(AgentResult {
            agent: d.agent,
            domain: d.domain,
            tasks,
            k3_true: tc,
            k3_unknown: uc,
            k3_false: fc,
            training_steps: steps,
            k3_true_pct: round_1((tc as f64) / (tasks as f64) * 100.0),
        });
        i += 1;
    }

    // python: execution_time_us = TASKS_PER_DAY / 1.05e12 * 1e6  (GPU IC, not measured here)
    let execution_time_us = round_i64((TASKS_PER_DAY as f64) / 1.05e12 * 1e6) as u64;

    DailyEvolutionReport {
        day: 1,
        total_tasks: TASKS_PER_DAY,
        agents: AGENTS,
        domains: DOMAINS.len(),
        k3: K3Report {
            true_count: total_true,
            true_pct: round_1((total_true as f64) / (TASKS_PER_DAY as f64) * 100.0),
            unknown_count: total_unknown,
            unknown_pct: round_1((total_unknown as f64) / (TASKS_PER_DAY as f64) * 100.0),
            false_count: total_false,
            false_pct: round_1((total_false as f64) / (TASKS_PER_DAY as f64) * 100.0),
        },
        training_steps: total_steps,
        execution_time_us,
        agent_results,
    }
}
