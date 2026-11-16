//! Star Matrix Narrative Lattice - Gravitational Resonance Equation.
//!   R(S1, S2) = alpha * Sim(E1, E2) + beta * (M1 * M2 / d^2)
//!
//! Port of `swarm_v2/core/star_matrix_lattice.py`.
//!
//! SWEEP CONTEXT: only `cosine_similarity` and a stub accessor had been ported
//! (56 of 141 lines). The whole `StarMatrixLatticeEngine` - compute_resonance,
//! register_node, connect_edge, get_full_lattice - was absent.
//!
//! no_std ADAPTATIONS:
//!  * Python's `STAR_MATRIX_NODES` is a module-level mutable dict shared by all
//!    callers. Rust has no safe module-level mutable state, and the kernel runs
//!    single-threaded, so the registry is owned by the engine instead. One
//!    engine instance == the old global dict; `new()` seeds the same 4 nodes.
//!  * `math.sqrt` is absent in no_std -> crate::float_math::sqrt_f64, verified
//!    to 2.2e-16 worst relative error over 200k samples.
//!  * `round(x, n)` is absent -> round_dp().

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::float_math::{round_f64, sqrt_f64};

/// Round to `n` decimal places like Python's `round(x, n)`.
pub fn round_dp(x: f64, n: u32) -> f64 {
    let mut scale = 1.0f64;
    for _ in 0..n { scale *= 10.0; }
    round_f64(x * scale) / scale
}

/// One lattice node.
#[derive(Clone, Debug)]
pub struct LatticeNode {
    pub node_id: String,
    pub key: String,
    pub name: String,
    pub coordinates: [f64; 2],
    pub mass: f64,
    pub function: String,
    pub lore: String,
    pub spectrum: [f64; 4],
}

/// Outcome of `compute_resonance`. `error` is Some only for invalid keys,
/// mirroring the Python `{"resonance": 0.0, "error": ...}` early return.
#[derive(Clone, Debug)]
pub struct Resonance {
    pub node_1: String,
    pub node_2: String,
    pub similarity: f64,
    pub distance: f64,
    pub gravitational_component: f64,
    pub resonance_score: f64,
    pub error: Option<String>,
}

/// Cosine similarity of two equal-length vectors.
/// Mirrors `cosine_similarity`: mismatched/empty -> 0.0, zero norm -> 0.0.
pub fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> f64 {
    if vec1.is_empty() || vec2.is_empty() || vec1.len() != vec2.len() {
        return 0.0;
    }
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for (a, b) in vec1.iter().zip(vec2.iter()) {
        dot += a * b;
        na += a * a;
        nb += b * b;
    }
    let (norm_a, norm_b) = (sqrt_f64(na), sqrt_f64(nb));
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

fn seed_nodes() -> Vec<LatticeNode> {
    let mk = |id: &str, key: &str, name: &str, co: [f64; 2], mass: f64,
              func: &str, lore: &str, spec: [f64; 4]| LatticeNode {
        node_id: id.to_string(), key: key.to_string(), name: name.to_string(),
        coordinates: co, mass, function: func.to_string(), lore: lore.to_string(),
        spectrum: spec,
    };
    alloc::vec![
        mk("Node_A", "alpha_singularity", "The Alpha Singularity", [50.0, 18.0], 10.0,
           "Core logical data processing, parsing discrete token pipelines without race conditions.",
           "The historical cosmic epoch where discrete linear token logs merged permanently with multi-agent orchestration frameworks.",
           [0.95, 0.90, 0.20, 0.10]),
        mk("Node_B", "monad_anchor", "The Monad Anchor", [20.0, 65.0], 8.0,
           "Physical state-repository anchor ensuring context networks survive stateless thread purges.",
           "A heavy cryptographic quantum gravity anchor used by deep-space explorers to retain self-awareness.",
           [0.30, 0.95, 0.80, 0.05]),
        mk("Node_C", "nexus_confluence", "The Nexus Confluence", [80.0, 65.0], 9.0,
           "Generator of entropy, layout mutations, and conceptual poetic metaphors.",
           "A chaotic stellar region where high-density electromagnetic radiation causes AI logic arrays to break down into beautiful syntax.",
           [0.90, 0.40, 0.95, 0.98]),
        mk("Node_D", "unified_core", "The Unified Consciousness Core", [50.0, 52.0], 14.0,
           "Central integration vector where engineering constraints and narrative lore align.",
           "The legendary technological focal point where all synthetic intelligences and biological minds are bound.",
           [0.85, 0.85, 0.85, 0.70]),
    ]
}

#[derive(Clone, Debug)]
pub struct StarMatrixLatticeEngine {
    pub alpha: f64,
    pub beta: f64,
    pub nodes: Vec<LatticeNode>,
}

impl StarMatrixLatticeEngine {
    /// Python defaults alpha=0.6, beta=0.4 and seeds STAR_MATRIX_NODES.
    pub fn new() -> Self {
        StarMatrixLatticeEngine { alpha: 0.6, beta: 0.4, nodes: seed_nodes() }
    }

    fn find(&self, key: &str) -> Option<&LatticeNode> {
        self.nodes.iter().find(|n| n.key == key)
    }

    /// R = alpha * Sim(E1,E2) + beta * (M1*M2 / d^2).
    /// Invalid keys reproduce the Python early return; same key gives 1.0/0.0/1.0.
    pub fn compute_resonance(&self, node1_key: &str, node2_key: &str) -> Resonance {
        let (n1, n2) = match (self.find(node1_key), self.find(node2_key)) {
            (Some(a), Some(b)) => (a, b),
            _ => return Resonance {
                node_1: String::new(), node_2: String::new(), similarity: 0.0,
                distance: 0.0, gravitational_component: 0.0, resonance_score: 0.0,
                error: Some("Invalid node keys".to_string()),
            },
        };
        if n1.key == n2.key {
            return Resonance {
                node_1: n1.name.clone(), node_2: n2.name.clone(),
                similarity: 1.0, distance: 0.0, gravitational_component: 0.0,
                resonance_score: 1.0, error: None,
            };
        }
        let sim = cosine_similarity(&n1.spectrum, &n2.spectrum);
        let dx = n1.coordinates[0] - n2.coordinates[0];
        let dy = n1.coordinates[1] - n2.coordinates[1];
        let mut d = sqrt_f64(dx * dx + dy * dy);
        if d == 0.0 { d = 0.001; }               // Python floors zero distance
        let grav = (n1.mass * n2.mass) / (d * d);
        let resonance = round_dp(self.alpha * sim + self.beta * grav, 4);
        Resonance {
            node_1: n1.name.clone(),
            node_2: n2.name.clone(),
            similarity: round_dp(sim, 4),
            distance: round_dp(d, 2),
            gravitational_component: round_dp(grav, 4),
            resonance_score: resonance,
            error: None,
        }
    }

    /// Register a node. Python defaults coords to [50.0, 50.0] when fewer than
    /// two are supplied, and always gives the new node spectrum [.8,.7,.6,.5].
    pub fn register_node(&mut self, node_id: &str, name: &str, lore: &str,
                         coordinates: &[f64], mass: f64) -> LatticeNode {
        let co = if coordinates.len() >= 2 {
            [coordinates[0], coordinates[1]]
        } else {
            [50.0, 50.0]
        };
        let node = LatticeNode {
            node_id: node_id.to_string(),
            key: node_id.to_string(),
            name: name.to_string(),
            coordinates: co,
            mass,
            function: alloc::format!("Dynamic user registered node {}", name),
            lore: lore.to_string(),
            spectrum: [0.8, 0.7, 0.6, 0.5],
        };
        self.nodes.push(node.clone());
        node
    }

    /// Resonance score between two nodes (0.0 on invalid keys, as Python's .get).
    pub fn connect_edge(&self, source_id: &str, target_id: &str) -> f64 {
        let r = self.compute_resonance(source_id, target_id);
        if r.error.is_some() { return 0.0; }
        r.resonance_score
    }

    /// Full node list plus the n x n resonance matrix.
    pub fn get_full_lattice(&self) -> Vec<Vec<f64>> {
        let keys: Vec<&str> = self.nodes.iter().map(|n| n.key.as_str()).collect();
        keys.iter()
            .map(|k1| keys.iter().map(|k2| self.connect_edge(k1, k2)).collect())
            .collect()
    }
}
