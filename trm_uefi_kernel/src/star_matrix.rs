//! TRM Swarm OS - Star Matrix Narrative Lattice (no_std Rust port).
//! 1:1 with swarm_v2/core/star_matrix.py:
//!   - Gravitational Resonance: R(S1,S2) = ALPHA*Sim(E1,E2) + BETA*(M1*M2)/d^2
//!   - cosine similarity (0.5 fallback on empty/mismatched/zero-norm),
//!     euclidean distance clamped >= 1.0
//!   - 4 canonical celestial nodes (alpha/monad/nexus/gamma), pairwise
//!     connection graph rounded to 6 decimals, D3 overlay export
//! Port spec validated by Devo via TRM bridge (agent_intelligence_bridge).

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const ALPHA: f64 = 0.6; // semantic similarity weight
pub const BETA: f64 = 0.4; // gravitational attraction weight
pub const SYNC_HISTORY_MAX: usize = 100;

/// A celestial node in the narrative lattice.
#[derive(Debug, Clone)]
pub struct StarMatrixNode {
    pub node_id: String,
    pub label: String,
    pub position: (f64, f64),
    pub mass: f64,
    pub embedding: Vec<f64>,
    pub linked_agents: Vec<String>,
    pub resonance_score: f64,
    pub last_sync_ns: u64,
}

impl StarMatrixNode {
    pub fn new(
        node_id: &str,
        label: &str,
        position: (f64, f64),
        mass: f64,
        linked_agents: Vec<String>,
        embedding: Vec<f64>,
        resonance_score: f64,
    ) -> Self {
        StarMatrixNode {
            node_id: node_id.into(),
            label: label.into(),
            position,
            mass,
            embedding,
            linked_agents,
            resonance_score,
            last_sync_ns: crate::time::now_ns(),
        }
    }
}

/// One pairwise connection with resonance rounded to 6 decimals (python round()).
#[derive(Debug, Clone)]
pub struct Connection {
    pub source: String,
    pub target: String,
    pub resonance: f64,
    pub timestamp_ns: u64,
}

/// D3.js-consumable node row.
#[derive(Debug, Clone)]
pub struct D3Node {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub resonance_score: f64,
    pub linked_agents: Vec<String>,
}

/// D3.js-consumable link row.
#[derive(Debug, Clone)]
pub struct D3Link {
    pub source: String,
    pub target: String,
    pub strength: f64,
}

pub struct StarMatrix {
    pub nodes: Vec<StarMatrixNode>,
    pub connections: Vec<Connection>,
    pub resonance_bound: bool,
    pub agent_mesh_bound: bool,
    pub sync_history_len: usize,
}

impl StarMatrix {
    pub fn new() -> Self {
        let mut m = StarMatrix {
            nodes: vec![],
            connections: vec![],
            resonance_bound: false,
            agent_mesh_bound: false,
            sync_history_len: 0,
        };
        m.init_default_nodes();
        m.compute_all_connections();
        m
    }

    /// Populate the lattice with the four canonical celestial nodes.
    fn init_default_nodes(&mut self) {
        let defaults: [(&str, &str, (f64, f64), f64, [&str; 2]); 4] = [
            ("alpha", "The Alpha Singularity", (50.0, 18.0), 10.0, ["Architect", "Lead Developer"]),
            ("monad", "The Monad Anchor", (20.0, 65.0), 8.0, ["Research Analyst", "Data Engineer"]),
            ("nexus", "The Nexus Confluence", (80.0, 65.0), 9.0, ["QA Engineer", "Security Auditor"]),
            ("gamma", "The Unified Consciousness Core", (50.0, 52.0), 14.0, ["Orchestrator", "DevOps Engineer"]),
        ];
        for (id, label, pos, mass, agents) in defaults {
            let node = StarMatrixNode::new(
                id,
                label,
                pos,
                mass,
                agents.iter().map(|a| String::from(*a)).collect(),
                Vec::new(),
                0.0,
            );
            self.nodes.push(node);
        }
    }

    /// Bind external subsystem references (resonance engine / agent mesh).
    pub fn bind(&mut self, resonance_engine: bool, agent_mesh: bool) {
        if resonance_engine {
            self.resonance_bound = true;
        }
        if agent_mesh {
            self.agent_mesh_bound = true;
        }
    }

    /// Cosine similarity between two vectors.
    /// 0.5 default when embeddings unavailable / mismatched / zero-norm.
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.5;
        }
        let mut dot = 0.0f64;
        let mut mag_a = 0.0f64;
        let mut mag_b = 0.0f64;
        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * y;
            mag_a += x * x;
            mag_b += y * y;
        }
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.5;
        }
        dot / (sqrt_f64(mag_a) * sqrt_f64(mag_b))
    }

    /// Euclidean distance between two 2-D points, clamped to >= 1.0.
    pub fn euclidean_distance(p1: (f64, f64), p2: (f64, f64)) -> f64 {
        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        sqrt_f64(dx * dx + dy * dy).max(1.0)
    }

    /// Gravitational Resonance Equation: R = ALPHA*Sim + BETA*(M1*M2)/d^2.
    /// Returns None (KeyError) if either node id is unknown.
    pub fn compute_gravitational_resonance(&self, node_a_id: &str, node_b_id: &str) -> Option<f64> {
        let node_a = self.nodes.iter().find(|n| n.node_id == node_a_id)?;
        let node_b = self.nodes.iter().find(|n| n.node_id == node_b_id)?;
        let sim = Self::cosine_similarity(&node_a.embedding, &node_b.embedding);
        let dist = Self::euclidean_distance(node_a.position, node_b.position);
        let gravitational = (node_a.mass * node_b.mass) / (dist * dist);
        Some(ALPHA * sim + BETA * gravitational)
    }

    /// Compute resonance for every unique pair (sorted ids) and cache result.
    pub fn compute_all_connections(&mut self) -> Vec<Connection> {
        let mut node_ids: Vec<String> = self.nodes.iter().map(|n| n.node_id.clone()).collect();
        node_ids.sort();
        let mut connections = Vec::new();
        for (i, id_a) in node_ids.iter().enumerate() {
            for id_b in node_ids.iter().skip(i + 1) {
                if let Some(resonance) = self.compute_gravitational_resonance(id_a, id_b) {
                    connections.push(Connection {
                        source: id_a.clone(),
                        target: id_b.clone(),
                        resonance: round_to(resonance, 6),
                        timestamp_ns: crate::time::now_ns(),
                    });
                }
            }
        }
        self.connections = connections.clone();
        connections
    }

    /// Full lattice state: (nodes, connections, sync_count).
    pub fn get_lattice(&self) -> (Vec<StarMatrixNode>, Vec<Connection>, usize) {
        (self.nodes.clone(), self.connections.clone(), self.sync_history_len)
    }

    /// D3.js-consumable representation of the lattice.
    pub fn get_d3_overlay(&self) -> (Vec<D3Node>, Vec<D3Link>) {
        let d3_nodes: Vec<D3Node> = self
            .nodes
            .iter()
            .map(|n| D3Node {
                id: n.node_id.clone(),
                label: n.label.clone(),
                x: n.position.0,
                y: n.position.1,
                mass: n.mass,
                resonance_score: n.resonance_score,
                linked_agents: n.linked_agents.clone(),
            })
            .collect();
        let d3_links: Vec<D3Link> = self
            .connections
            .iter()
            .map(|c| D3Link {
                source: c.source.clone(),
                target: c.target.clone(),
                strength: c.resonance,
            })
            .collect();
        (d3_nodes, d3_links)
    }
}

/// Python-style round(x, n): round-half-to-even on the scaled value.
fn round_to(x: f64, n: u32) -> f64 {
    let p = pow10_f64(n);
    round_ties_even(x * p) / p
}

fn pow10_f64(n: u32) -> f64 {
    let mut r = 1.0f64;
    for _ in 0..n {
        r *= 10.0;
    }
    r
}

/// round-half-to-even (banker's rounding, matches python builtin round()).
fn round_ties_even(x: f64) -> f64 {
    let f = floor_f64(x);
    let diff = x - f;
    if diff > 0.5 {
        f + 1.0
    } else if diff < 0.5 {
        f
    } else {
        let fl = f as i64;
        if fl % 2 == 0 {
            f
        } else {
            f + 1.0
        }
    }
}

fn floor_f64(x: f64) -> f64 {
    let i = x as i64;
    if x >= 0.0 || x == i as f64 {
        i as f64
    } else {
        (i - 1) as f64
    }
}

/// no_std f64 sqrt - Newton-Raphson from bit-hack seed (matches math.sqrt to ~1 ulp).
fn sqrt_f64(v: f64) -> f64 {
    if v <= 0.0 {
        return 0.0;
    }
    let bits = v.to_bits();
    let seed_bits = (bits.wrapping_add(1023u64 << 52) >> 1) as u64;
    let mut x = f64::from_bits(seed_bits);
    for _ in 0..6 {
        x = 0.5 * (x + v / x);
    }
    x
}
