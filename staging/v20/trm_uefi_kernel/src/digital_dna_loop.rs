//! TRM Swarm OS - digital_dna_loop (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/digital_dna_loop.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;


//  CognitiveGene 
#[derive(Debug, Clone)]
pub struct CognitiveGene {
    pub plasticity: f64,
    pub logical_depth: f64,
    pub empathy: f64,
    pub stochasticity: f64,
}

impl CognitiveGene {
    pub fn new() -> Self {
        CognitiveGene {
            plasticity: 0.0,
            logical_depth: 0.0,
            empathy: 0.0,
            stochasticity: 0.0,
        }
    }
}

//  DNAMutationProposal 
#[derive(Debug, Clone)]
pub struct DNAMutationProposal {
    pub proposal_id: String,
    pub source_role: String,
    pub gene_before: u64,
    pub gene_after: u64,
    pub reason: String,
    pub timestamp: String,
    pub status: String,
}

impl DNAMutationProposal {
    pub fn new() -> Self {
        DNAMutationProposal {
            proposal_id: String::new(),
            source_role: String::new(),
            gene_before: 0,
            gene_after: 0,
            reason: String::new(),
            timestamp: String::new(),
            status: String::new(),
        }
    }
}

//  get_digital_dna_loop() 
static SINGLETON_DIGITAL_DNA_LOOP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_digital_dna_loop() -> u64 {
    // Singleton handle (lazy-init pattern from digital_dna_loop.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_DIGITAL_DNA_LOOP.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_DIGITAL_DNA_LOOP.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's model (swarm_v2/core/digital_dna_loop.py + strip_digital_dna.rs) ────────────────
//
// `native_gene_drift(history_matrix, preset_vec)`: for a history of `count` rows of FOUR gene metrics
// (rho plasticity, delta logical_depth, epsilon empathy, sigma stochasticity), the per-metric average,
// plus `drift = sqrt(sum_i (avg_i - preset_i)^2)` -- the strip's `strip_cognitive_gene_drift_evaluator`,
// in f32, which is what the port is checked against.
//
// `evaluate_performance(role, error_rate, avg_latency)`: three rules over the gene, each assignment
// clamped to [0, 1]:
//     error_rate > 0.15 -> sigma - 0.1, delta + 0.1
//     avg_latency > 5.0 -> rho + 0.15
//     no reason yet AND error_rate < 0.02 -> sigma + 0.05 (exploration reward)
// with no rule firing at all returning None. The reason strings carry the oracle's formatting, three
// decimals for the error rate and two for the latency.

/// `_clamp`: `max(0.0, min(1.0, v))`.
pub fn clamp(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

/// The four gene metrics, in the oracle's order: (plasticity, logical_depth, empathy, stochasticity).
pub type Gene = [f32; 4];

/// `native_gene_drift`: (per-metric averages, drift). `history` is `count * 4` values, row-major.
pub fn native_gene_drift(history: &[f32], preset: &Gene) -> (Gene, f32) {
    let count = history.len() / 4;
    let mut sums = [0.0f32; 4];
    if count > 0 {
        for i in 0..count {
            let base = i * 4;
            sums[0] += history[base];
            sums[1] += history[base + 1];
            sums[2] += history[base + 2];
            sums[3] += history[base + 3];
        }
    }
    let inv_n = 1.0f32 / (count as f32);
    let mut avg = [0.0f32; 4];
    let mut sum_sq = 0.0f32;
    for i in 0..4 {
        let a = sums[i] * inv_n;
        avg[i] = a;
        let diff = a - preset[i];
        sum_sq += diff * diff;
    }
    (avg, sqrt_f32(sum_sq))
}

/// `sqrt` without std (the f64 form, rounded once), as elsewhere in this tree.
pub fn sqrt_f32(val: f32) -> f32 {
    if val <= 0.0 {
        return 0.0;
    }
    let v = val as f64;
    let mut x = v;
    for _ in 0..10 {
        x = 0.5 * (x + v / x);
    }
    x as f32
}

/// `evaluate_performance`: (the mutated gene, the reason) or None when no rule fires.
pub fn evaluate_performance(gene: &Gene, error_rate: f32, avg_latency: f32) -> Option<(Gene, String)> {
    let mut mutated = *gene;
    let mut reasons: Vec<String> = Vec::new();
    if error_rate > 0.15 {
        mutated[3] = clamp(mutated[3] - 0.1);
        mutated[1] = clamp(mutated[1] + 0.1);
        reasons.push(format!("high error_rate ({:.3}>0.15): sigma-0.1, delta+0.1", error_rate));
    }
    if avg_latency > 5.0 {
        mutated[0] = clamp(mutated[0] + 0.15);
        reasons.push(format!("high avg_latency ({:.2}s>5.0s): rho+0.15", avg_latency));
    }
    if reasons.is_empty() && error_rate < 0.02 {
        mutated[3] = clamp(mutated[3] + 0.05);
        reasons.push(format!("excellent error_rate ({:.3}<0.02): sigma+0.05 (exploration reward)", error_rate));
    }
    if reasons.is_empty() {
        return None;
    }
    Some((mutated, reasons.join("; ")))
}

// ── `CognitiveGene`, the system presets, and the resonant semantic cache ─────────────────────────
//
// `__post_init__` clamps all four metrics to [0, 1] (the `clamp` above), `to_vector`/`from_vector` are
// the flat [rho, delta, epsilon, sigma] order, and `distance` is the UNWEIGHTED Euclidean distance.
//
// The cache is the interesting part: a window of at most 50 genes (the oracle uses
// `deque(maxlen=50)`, so a full window drops the OLDEST -- a ring buffer, not a shift), a running
// average recomputed on every record, and drift measured from the "continuum" preset. Two rules that
// are easy to get wrong and are reproduced exactly here:
//   * `_recalculate_average` RETURNS EARLY on an empty window, so the average STAYS at whatever it was
//     (the constructor seeds it with continuum) rather than becoming zeros;
//   * `detect_drift` is a STRICT `>` against the threshold.
// `drift_from_continuum` is `round(distance, 6)`, so the rounding is half-to-even like python's.

/// The oracle's `SYSTEM_PRESETS`, verbatim: (name, [plasticity, logical_depth, empathy, stochasticity]).
pub const SYSTEM_PRESETS: [(&str, Gene); 4] = [
    ("continuum", [0.7, 0.8, 0.6, 0.5]),
    ("sage", [0.4, 0.95, 0.7, 0.2]),
    ("muse", [0.9, 0.5, 0.8, 0.9]),
    ("sentinel", [0.3, 0.9, 0.3, 0.1]),
];

fn continuum() -> Gene {
    SYSTEM_PRESETS[0].1
}

/// The oracle's preset lookup, defaulting to continuum.
pub fn system_preset(name: &str) -> Gene {
    for p in SYSTEM_PRESETS.iter() {
        if p.0 == name {
            return p.1;
        }
    }
    continuum()
}

/// `CognitiveGene.distance`: unweighted Euclidean distance over the four metrics.
///
/// In **f64**, because the oracle's `distance` returns a Python float and `round(d, 6)` is then an f64
/// operation. The components it reads are f32-valued (they come from the strip's kernel), so an f32
/// accumulator here cost the last digit of the reported drift: 0.905538 against the oracle's 0.905539.
/// Computing in f64 and rounding once is the same rule that made `sqrt` and the trig pair bit-exact.
pub fn distance(a: &Gene, b: &Gene) -> f64 {
    let mut sum = 0.0f64;
    for i in 0..4 {
        let d = (a[i] as f64) - (b[i] as f64);
        sum += d * d;
    }
    sqrt_f64(sum)
}

fn sqrt_f64(v: f64) -> f64 {
    if v <= 0.0 {
        return 0.0;
    }
    let mut x = v;
    for _ in 0..16 {
        x = 0.5 * (x + v / x);
    }
    x
}

/// `__post_init__`: clamp every metric into [0, 1].
pub fn clamp_gene(g: &Gene) -> Gene {
    [clamp(g[0]), clamp(g[1]), clamp(g[2]), clamp(g[3])]
}

fn floor_f64(v: f64) -> f64 {
    let t = v as i64 as f64;
    if v < 0.0 && t != v {
        t - 1.0
    } else {
        t
    }
}

/// Python's `round(x, 6)` for the reported drift -- half-to-even, via f64.
fn round6(x: f64) -> f64 {
    let scaled = x * 1_000_000.0;
    let fl = floor_f64(scaled);
    let diff = scaled - fl;
    let rounded = if diff > 0.5 {
        fl + 1.0
    } else if diff < 0.5 {
        fl
    } else if (fl / 2.0) == floor_f64(fl / 2.0) {
        fl
    } else {
        fl + 1.0
    };
    rounded / 1_000_000.0
}

/// `ResonantSemanticCache`: a 50-gene window, the running average, and drift from continuum.
pub struct GeneCache {
    history: [Gene; 50],
    count: usize,
    head: usize,
    pub average: Gene,
}

impl GeneCache {
    /// The constructor seeds the average with the continuum preset, as the oracle does.
    pub fn new() -> Self {
        GeneCache { history: [[0.0; 4]; 50], count: 0, head: 0, average: continuum() }
    }

    /// `record`: append (dropping the oldest when full) and recalculate.
    pub fn record(&mut self, g: Gene) {
        let g = clamp_gene(&g);
        if self.count < 50 {
            self.history[self.count] = g;
            self.count += 1;
        } else {
            self.history[self.head] = g;
            self.head = (self.head + 1) % 50;
        }
        self.recalculate_average();
    }

    /// The window in insertion order, oldest first.
    pub fn window(&self) -> Vec<Gene> {
        let mut out: Vec<Gene> = Vec::with_capacity(self.count);
        if self.count < 50 {
            for i in 0..self.count {
                out.push(self.history[i]);
            }
        } else {
            for i in 0..50 {
                out.push(self.history[(self.head + i) % 50]);
            }
        }
        out
    }

    /// `_recalculate_average` -- through the strip's own kernel, with the oracle's early return on an
    /// empty window.
    pub fn recalculate_average(&mut self) {
        if self.count == 0 {
            return;
        }
        let flat: Vec<f32> = self.window().iter().flat_map(|g| g.iter().copied()).collect();
        let (avg, _drift) = native_gene_drift(&flat, &continuum());
        self.average = clamp_gene(&avg);
    }

    /// `get_get_refraction_context`: (average gene, window size, drift rounded to 6).
    pub fn get_refraction_context(&self) -> (Gene, usize, f64) {
        (self.average, self.count, round6(distance(&self.average, &continuum())))
    }

    /// `detect_drift`: the raw distance (for reporting) and whether it STRICTLY exceeds the threshold.
    pub fn detect_drift(&self, threshold: f64) -> (f64, bool) {
        let drift = distance(&self.average, &continuum());
        (drift, drift > threshold)
    }
}

// ── The proposal store, for `apply_mutation`'s already-applied rule ───────────────────────────────

pub struct MutationProposal {
    pub proposal_id: u32,
    pub source_role: u32,
    pub gene_after: Gene,
    pub applied: bool,
}

/// The oracle's `_agent_genes` is a DICT keyed by role, and `apply_mutation` assigns into it -- so a
/// role that is not present is CREATED, not skipped. A fixed slot table has to reproduce that, which is
/// why this is a store with an insert rather than a lookup-and-update.
pub struct GeneStore {
    slots: [(u32, Gene); 8],
    n: usize,
}

impl GeneStore {
    pub fn new() -> Self {
        GeneStore { slots: [(0, [0.0; 4]); 8], n: 0 }
    }
    /// Dict-style assignment: update in place, or create the entry.
    pub fn set(&mut self, role: u32, gene: Gene) {
        for i in 0..self.n {
            if self.slots[i].0 == role {
                self.slots[i].1 = gene;
                return;
            }
        }
        if self.n < 8 {
            self.slots[self.n] = (role, gene);
            self.n += 1;
        }
    }
    pub fn get(&self, role: u32) -> Option<Gene> {
        for i in 0..self.n {
            if self.slots[i].0 == role {
                return Some(self.slots[i].1);
            }
        }
        None
    }
    pub fn len(&self) -> usize {
        self.n
    }
}

/// `apply_mutation`: already applied -> false (the oracle logs and skips), not found -> false, else the
/// role's gene becomes `gene_after` and the status flips to applied.
pub fn apply_mutation(genes: &mut GeneStore, proposals: &mut [MutationProposal], proposal_id: u32) -> bool {
    for p in proposals.iter_mut() {
        if p.proposal_id == proposal_id {
            if p.applied {
                return false;
            }
            genes.set(p.source_role, p.gene_after);
            p.applied = true;
            return true;
        }
    }
    false
}
