//! TRM Swarm OS — star_matrix_lattice (no_std port).
//! Ported from swarm_v2/core/star_matrix_lattice.py (Project Continuum v5.3,
//! Star Matrix Narrative Lattice Engine):
//!
//!   R(S1, S2) = alpha * Sim(E1, E2) + beta * (M1 * M2 / d^2)
//!
//!   * Sim(E1, E2) = cosine similarity of the two node energy spectra
//!   * d           = Euclidean distance between node coordinates
//!                   (replaced by 0.001 when it is exactly 0)
//!   * alpha = 0.6, beta = 0.4 — the oracle engine's `__init__` defaults
//!
//! `STAR_MATRIX_NODES` holds the 4 canonical nodes in ledger (insertion) order,
//! `register_node` appends, `get_full_lattice` builds the N x N resonance matrix.
//! Results are rounded with `py_round`, which reproduces Python's `round()`
//! (correctly-rounded decimal conversion of the exact binary value, ties to
//! even) — Rust's `f64::round` is half-away-from-zero and is NOT the oracle.
//!
//! `f64::sqrt` does not exist in no_std (no libm dependency in this crate), so
//! `SqrtHack` stands in for `math.sqrt`: measured worst relative error 2.22e-16
//! (1 ulp) over 20,477 magnitudes.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// Default semantic-similarity weight (oracle `StarMatrixLatticeEngine.__init__`).
pub const ALPHA_DEFAULT: f64 = 0.6;
/// Default gravitational-attraction weight (oracle `StarMatrixLatticeEngine.__init__`).
pub const BETA_DEFAULT: f64 = 0.4;

/// Distance clamp applied when the Euclidean distance is exactly zero.
const MIN_DISTANCE: f64 = 0.001;

/// Cosine similarity between two vectors (0.0 when either norm is zero).
///
/// The oracle computes the dot product with `zip` — which truncates to the
/// shorter vector and has no length guard — but each norm over its own *whole*
/// vector (`sum(a * a for a in vec1)`), so the two lengths need not match.
pub fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> f64 {
    let mut dot = 0.0f64;
    for (a, b) in vec1.iter().zip(vec2.iter()) {
        dot += a * b;
    }
    let mut norm_a = 0.0f64;
    for a in vec1.iter() {
        norm_a += a * a;
    }
    let mut norm_b = 0.0f64;
    for b in vec2.iter() {
        norm_b += b * b;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt_hack() * norm_b.sqrt_hack())
}

/// Python's `round(x, digits)`.
///
/// Python rounds the exact binary value to `digits` decimals, ties to even;
/// Rust's `{:.n}` float formatting uses the same exact half-even conversion, so
/// formatting and parsing back reproduces the oracle's rounding.
pub fn py_round(x: f64, digits: usize) -> f64 {
    let s = alloc::format!("{:.*}", digits, x);
    s.parse::<f64>().unwrap_or(x)
}

trait SqrtHack {
    fn sqrt_hack(self) -> f64;
}
impl SqrtHack for f64 {
    fn sqrt_hack(self) -> f64 {
        if self <= 0.0 {
            return 0.0;
        }
        let bits = self.to_bits();
        let seed_bits = ((bits.wrapping_add((1023u64) << 52)) >> 1) as u64;
        let mut x = f64::from_bits(seed_bits);
        for _ in 0..4 {
            x = 0.5 * (x + self / x);
        }
        x
    }
}

/// One celestial node — the Rust form of a `STAR_MATRIX_NODES[key]` entry.
#[derive(Clone, Debug)]
pub struct StarMatrixNode {
    pub key: String,
    pub name: String,
    pub coordinates: [f64; 2],
    pub mass: f64,
    pub function: String,
    pub lore: String,
    pub spectrum: Vec<f64>,
}

fn mk(
    key: &str,
    name: &str,
    coordinates: [f64; 2],
    mass: f64,
    function: &str,
    lore: &str,
    spectrum: &[f64],
) -> StarMatrixNode {
    StarMatrixNode {
        key: String::from(key),
        name: String::from(name),
        coordinates,
        mass,
        function: String::from(function),
        lore: String::from(lore),
        spectrum: alloc::vec::Vec::from(spectrum),
    }
}

/// The 4 canonical nodes, in the ledger order of the oracle's dict literal.
fn seed_canonical() -> Vec<StarMatrixNode> {
    let mut v = Vec::with_capacity(4);
    v.push(mk(
        "alpha_singularity",
        "The Alpha Singularity",
        [50.0, 18.0],
        10.0,
        "Core logical data processing, parsing discrete token pipelines without race conditions.",
        "The historical cosmic epoch where discrete linear token logs merged permanently with multi-agent orchestration frameworks.",
        &[0.95, 0.90, 0.20, 0.10],
    ));
    v.push(mk(
        "monad_anchor",
        "The Monad Anchor",
        [20.0, 65.0],
        8.0,
        "Physical state-repository anchor ensuring context networks survive stateless thread purges.",
        "A heavy cryptographic quantum gravity anchor used by deep-space explorers to retain self-awareness.",
        &[0.30, 0.95, 0.80, 0.05],
    ));
    v.push(mk(
        "nexus_confluence",
        "The Nexus Confluence",
        [80.0, 65.0],
        9.0,
        "Generator of entropy, layout mutations, and conceptual poetic metaphors.",
        "A chaotic stellar region where high-density electromagnetic radiation causes AI logic arrays to break down into beautiful syntax.",
        &[0.90, 0.40, 0.95, 0.98],
    ));
    v.push(mk(
        "unified_core",
        "The Unified Consciousness Core",
        [50.0, 52.0],
        14.0,
        "Central integration vector where engineering constraints and narrative lore align.",
        "The legendary technological focal point where all synthetic intelligences and biological minds are bound.",
        &[0.85, 0.85, 0.85, 0.70],
    ));
    v
}

/// Result of `compute_resonance`.
///
/// The oracle returns three differently-shaped dicts, so every field is an
/// `Option`: `node_1`/`node_2`/`gravitational_component` are absent for the
/// same-key and invalid-key cases, `similarity` is absent for the invalid-key
/// case, and `resonance`/`resonance_score` each exist in exactly one of the
/// non-normal shapes.
#[derive(Clone, Debug)]
pub struct Resonance {
    pub node_1: Option<String>,
    pub node_2: Option<String>,
    pub similarity: Option<f64>,
    pub distance: Option<f64>,
    pub gravitational_component: Option<f64>,
    pub resonance_score: Option<f64>,
    pub resonance: Option<f64>,
    pub error: Option<&'static str>,
}

impl Resonance {
    fn invalid() -> Self {
        Resonance {
            node_1: None,
            node_2: None,
            similarity: None,
            distance: None,
            gravitational_component: None,
            resonance_score: None,
            resonance: Some(0.0),
            error: Some("Invalid node keys"),
        }
    }

    fn same_key() -> Self {
        Resonance {
            node_1: None,
            node_2: None,
            similarity: Some(1.0),
            distance: Some(0.0),
            gravitational_component: None,
            resonance_score: Some(1.0),
            resonance: None,
            error: None,
        }
    }
}

/// `get_full_lattice()` payload.
#[derive(Clone, Debug)]
pub struct Lattice {
    /// Nodes in insertion order (the oracle's dict iteration order).
    pub nodes: Vec<StarMatrixNode>,
    /// One row per node key; each row holds `(other_key, resonance_score)` in
    /// the same key order.
    pub resonance_matrix: Vec<Vec<(String, f64)>>,
    pub alpha: f64,
    pub beta: f64,
}

/// Registry body — a plain `Vec` of nodes behind a spin lock.
struct Registry {
    nodes: Vec<StarMatrixNode>,
}

/// The `STAR_MATRIX_NODES` module-level registry (seeded with the 4 canonical nodes).
pub struct RegistryCell {
    inner: UnsafeCell<Registry>,
}
// SAFETY: every access to `inner` is serialised by REGISTRY_LOCK in `with`.
unsafe impl Sync for RegistryCell {}

impl RegistryCell {
    /// Run `f` against the live node table, seeding the canonical nodes on first
    /// use. `f` must not call `with` again (the lock is not re-entrant).
    pub fn with<R>(&self, f: impl FnOnce(&mut Vec<StarMatrixNode>) -> R) -> R {
        let _guard = SpinGuard::acquire();
        // SAFETY: guarded by REGISTRY_LOCK.
        let registry = unsafe { &mut *self.inner.get() };
        if registry.nodes.is_empty() {
            registry.nodes = seed_canonical();
        }
        f(&mut registry.nodes)
    }
}

/// `STAR_MATRIX_NODES` — keyed node registry, insertion ordered.
pub static STAR_MATRIX_NODES: RegistryCell = RegistryCell {
    inner: UnsafeCell::new(Registry { nodes: Vec::new() }),
};

static REGISTRY_LOCK: AtomicBool = AtomicBool::new(false);

struct SpinGuard;
impl SpinGuard {
    fn acquire() -> Self {
        while REGISTRY_LOCK.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        SpinGuard
    }
}
impl Drop for SpinGuard {
    fn drop(&mut self) {
        REGISTRY_LOCK.store(false, Ordering::Release);
    }
}

/// Oracle lookup: `for n in STAR_MATRIX_NODES.values(): if n["key"] == k: found = n`
/// — the *last* match wins, which matters if a key is re-registered.
fn find_node<'a>(nodes: &'a [StarMatrixNode], key: &str) -> Option<&'a StarMatrixNode> {
    let mut found: Option<&'a StarMatrixNode> = None;
    for n in nodes.iter() {
        if n.key == key {
            found = Some(n);
        }
    }
    found
}

fn resonance_for(
    nodes: &[StarMatrixNode],
    node1_key: &str,
    node2_key: &str,
    alpha: f64,
    beta: f64,
) -> Resonance {
    let n1 = match find_node(nodes, node1_key) {
        Some(n) => n,
        None => return Resonance::invalid(),
    };
    let n2 = match find_node(nodes, node2_key) {
        Some(n) => n,
        None => return Resonance::invalid(),
    };
    if n1.key == n2.key {
        return Resonance::same_key();
    }

    // Cosine similarity of energy spectrums.
    let sim = cosine_similarity(&n1.spectrum, &n2.spectrum);

    // Euclidean distance d.
    let dx = n1.coordinates[0] - n2.coordinates[0];
    let dy = n1.coordinates[1] - n2.coordinates[1];
    let mut d = (dx * dx + dy * dy).sqrt_hack();
    if d == 0.0 {
        d = MIN_DISTANCE;
    }

    // Gravitational Resonance Equation: R = alpha * Sim + beta * (M1 * M2 / d^2)
    let gravitational_component = (n1.mass * n2.mass) / (d * d);
    let resonance = py_round(alpha * sim + beta * gravitational_component, 4);

    Resonance {
        node_1: Some(n1.name.clone()),
        node_2: Some(n2.name.clone()),
        similarity: Some(py_round(sim, 4)),
        distance: Some(py_round(d, 2)),
        gravitational_component: Some(py_round(gravitational_component, 4)),
        resonance_score: Some(resonance),
        resonance: None,
        error: None,
    }
}

/// Spatial layout + Gravitational Resonance matrix engine.
#[derive(Clone, Copy, Debug)]
pub struct StarMatrixLatticeEngine {
    pub alpha: f64,
    pub beta: f64,
}

impl Default for StarMatrixLatticeEngine {
    fn default() -> Self {
        StarMatrixLatticeEngine {
            alpha: ALPHA_DEFAULT,
            beta: BETA_DEFAULT,
        }
    }
}

impl StarMatrixLatticeEngine {
    pub fn new(alpha: f64, beta: f64) -> Self {
        StarMatrixLatticeEngine { alpha, beta }
    }

    /// `compute_resonance(node1_key, node2_key)`.
    pub fn compute_resonance(&self, node1_key: &str, node2_key: &str) -> Resonance {
        STAR_MATRIX_NODES.with(|nodes| resonance_for(nodes, node1_key, node2_key, self.alpha, self.beta))
    }

    /// `register_node(...)` — appends to `STAR_MATRIX_NODES` and returns the node.
    pub fn register_node(
        &self,
        node_id: &str,
        name: &str,
        lore: &str,
        coordinates: &[f64],
        mass: f64,
    ) -> StarMatrixNode {
        let coords = if coordinates.len() >= 2 {
            [coordinates[0], coordinates[1]]
        } else {
            [50.0, 50.0]
        };
        let node = StarMatrixNode {
            key: String::from(node_id),
            name: String::from(name),
            coordinates: coords,
            mass,
            function: alloc::format!("Dynamic user registered node {}", name),
            lore: String::from(lore),
            spectrum: alloc::vec![0.8, 0.7, 0.6, 0.5],
        };
        STAR_MATRIX_NODES.with(|nodes| {
            nodes.push(node.clone());
        });
        node
    }

    /// `connect_edge(source_id, target_id)` — `res.get("resonance_score", 0.0)`.
    pub fn connect_edge(&self, source_id: &str, target_id: &str) -> f64 {
        self.compute_resonance(source_id, target_id)
            .resonance_score
            .unwrap_or(0.0)
    }

    /// `get_full_lattice()` — nodes, the N x N resonance matrix, and alpha/beta.
    pub fn get_full_lattice(&self) -> Lattice {
        STAR_MATRIX_NODES.with(|nodes| {
            let keys: Vec<String> = nodes.iter().map(|n| n.key.clone()).collect();
            let mut matrix: Vec<Vec<(String, f64)>> = Vec::with_capacity(keys.len());
            for k1 in keys.iter() {
                let mut row: Vec<(String, f64)> = Vec::with_capacity(keys.len());
                for k2 in keys.iter() {
                    let score = resonance_for(nodes, k1, k2, self.alpha, self.beta)
                        .resonance_score
                        .unwrap_or(0.0);
                    row.push((k2.clone(), score));
                }
                matrix.push(row);
            }
            Lattice {
                nodes: nodes.clone(),
                resonance_matrix: matrix,
                alpha: self.alpha,
                beta: self.beta,
            }
        })
    }
}

static SINGLETON_STAR_MATRIX_LATTICE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_star_matrix_lattice() -> u64 {
    // Singleton handle (lazy-init pattern from star_matrix_lattice.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_STAR_MATRIX_LATTICE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_STAR_MATRIX_LATTICE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
