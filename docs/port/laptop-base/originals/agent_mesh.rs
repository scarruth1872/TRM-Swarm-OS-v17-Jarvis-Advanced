//! TRM Swarm OS - agent_mesh (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/agent_mesh.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MESH_DIR: &'static str = "swarm_v2_mesh";

//  MeshNodeSchema 
#[derive(Debug, Clone)]
pub struct MeshNodeSchema {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub host: String,
    pub port: i64,
    pub specialties: Vec<String>,
    pub skills: Vec<String>,
    pub hardware: Vec<(String, f64)>,
    pub status: String,
    pub last_heartbeat: f64,
    pub joined_at: String,
    pub task_count: i64,
    pub peer_connections: i64,
}

// ── The oracle's model (swarm_v2/core/agent_mesh.py + strip_agent_mesh.rs) ────────────────────────
//
// `_native_mesh_distance_scorer` is a pure routing score, identical in the python fallback and in the
// module's strip source (`swarm_v2_artifacts/native_voltage_strips/strip_agent_mesh.rs`):
//     dist   = sqrt(dx*dx + dy*dy + dz*dz)
//     score  = dist * 0.4 + latency * 0.6
// with f32 throughout. The module carried only the node schema and none of this computation.
//
// `sqrt_f32` mirrors the helper the rest of this tree uses (`cognitive_ingestion_engine.rs`,
// `collaborative_reasoning.rs`, `manus_engine.rs`) because bare-metal no_std has no `f32::sqrt`.
// Newton's method is an APPROXIMATION of the correctly-rounded result a strip's std sqrt produces,
// so the port is verified against `strip_agent_mesh.dll` itself and any last-bit difference is
// measured rather than assumed away.

/// Newton's method in **f64**, then one rounding to f32 -- which is what makes the result the
/// correctly-rounded f32 one. Doing the same iteration in f32 leaves the fixed point 1-2 ULP off:
/// measured against `strip_agent_mesh.dll` over 282 distances, f32 Newton differs on **66 (23.4%)**
/// with a worst case of 2 ULP, while f64 Newton differs on **0**. Extra f32 iterations do not help
/// (10, 12, 16, 20 and 24 all give the same 66), so this is the method, not the convergence.
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

/// Per-node routing scores for a target point: `dist * 0.4 + latency * 0.6`. `coords` is 3 per node.
pub fn mesh_distance_scores(coords: &[f32], latencies: &[f32], target: &[f32; 3]) -> Vec<f32> {
    let nodes = latencies.len();
    let mut out: Vec<f32> = Vec::with_capacity(nodes);
    for i in 0..nodes {
        let b = i * 3;
        if b + 3 > coords.len() {
            break;
        }
        let dx = coords[b] - target[0];
        let dy = coords[b + 1] - target[1];
        let dz = coords[b + 2] - target[2];
        let dist = sqrt_f32(dx * dx + dy * dy + dz * dz);
        out.push(dist * 0.4 + latencies[i] * 0.6);
    }
    out
}

/// The winning node (lowest score), as the router chooses one destination. Returns None for an
/// empty mesh rather than node 0, which would be a fabricated choice.
pub fn best_node(coords: &[f32], latencies: &[f32], target: &[f32; 3]) -> Option<(usize, f32)> {
    let scores = mesh_distance_scores(coords, latencies, target);
    if scores.is_empty() {
        return None;
    }
    let mut bi = 0usize;
    let mut bs = scores[0];
    for (i, s) in scores.iter().enumerate() {
        if *s < bs {
            bs = *s;
            bi = i;
        }
    }
    Some((bi, bs))
}

// ── the mesh graph rules (swarm_v2/core/agent_mesh.py) ───────────────────────────────────────────
//
// `_compute_connections` walks every unordered node pair and scores it with three heuristics:
//     shared specialties: +2 per shared specialty
//     role proximity:     +1 when BOTH roles contain one of {developer, architect, engineer, logic}
//     architect star:     +1 when either role EXACTLY equals "Architect"
// and emits the pair only when the total is above zero. Note the asymmetry, which is the oracle's:
// the proximity test is a substring search on the lowercased role, while the star test is an exact
// string comparison.
//
// `cleanup_stale_nodes` removes nodes whose `now - last_heartbeat` exceeds the timeout -- strictly
// greater, so a node exactly at the timeout survives -- and bumps `topology_version` by one when it
// removed anything. The clock arrives as an argument; firmware has no wall clock in this path.

/// One node as these rules need it. `compute_connections` reads only `node_id`/`role`/
/// `specialties`; the routing methods (`find_best_node`, `reconfigure_mesh`, `get_stats`)
/// also read `skills`, `task_count`, and the derived `is_alive`.
#[derive(Debug, Clone)]
pub struct MeshNode {
    pub node_id: String,
    pub role: String,
    pub specialties: Vec<String>,
    pub skills: Vec<String>,
    pub task_count: f64,
    pub is_alive: bool,
}

/// A scored pair, as `_compute_connections` returns it.
#[derive(Debug, Clone)]
pub struct MeshConnection {
    pub from: String,
    pub to: String,
    pub shared_specialties: Vec<String>,
    pub strength: u64,
}

/// The oracle's three heuristics over every unordered pair, in the node order given.
pub fn compute_connections(nodes: &[MeshNode]) -> Vec<MeshConnection> {
    let dev_words = ["developer", "architect", "engineer", "logic"];
    let mut out: Vec<MeshConnection> = Vec::new();
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let a = &nodes[i];
            let b = &nodes[j];
            if a.node_id == b.node_id {
                continue;
            }
            let mut shared: Vec<String> = Vec::new();
            for s in a.specialties.iter() {
                if b.specialties.contains(s) && !shared.contains(s) {
                    shared.push(s.clone());
                }
            }
            let mut strength = shared.len() as u64 * 2;
            let al = a.role.to_lowercase();
            let bl = b.role.to_lowercase();
            if dev_words.iter().any(|w| al.contains(w)) && dev_words.iter().any(|w| bl.contains(w)) {
                strength += 1;
            }
            if a.role == "Architect" || b.role == "Architect" {
                strength += 1;
            }
            if strength > 0 {
                out.push(MeshConnection {
                    from: a.node_id.clone(),
                    to: b.node_id.clone(),
                    shared_specialties: shared,
                    strength,
                });
            }
        }
    }
    out
}

/// `cleanup_stale_nodes`, named as the oracle names it: the node ids it would remove, in input order.
/// The caller bumps its topology version when this is non-empty, which is the oracle's condition too;
/// firmware holds no node map here, so the removal itself stays with the caller.
pub fn cleanup_stale_nodes(nodes: &[(String, f64)], now: f64, timeout: f64) -> Vec<String> {
    nodes
        .iter()
        .filter(|(_, hb)| (now - *hb) > timeout)
        .map(|(id, _)| id.clone())
        .collect()
}

/// `find_best_node`: the routing score. +10 per specialty match, +5 per skill match, +8 when
/// the role appears in the task, +20 when the required specialty is present (nodes without it
/// are skipped), −0.1 × task_count. The top positive scorer wins; a tie keeps the first in
/// input order (the oracle's stable sort). When no node scores above zero the oracle falls back
/// to the first alive node; when the top score is under 10 it would try the semantic embedder,
/// which is host-side — reported via `needs_semantic` rather than faked.
#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub node_id: Option<String>,
    pub score: f64,
    pub needs_semantic: bool,
}

pub fn find_best_node(
    nodes: &[MeshNode],
    task: &str,
    required_specialty: Option<&str>,
) -> RoutingResult {
    let task_lower = task.to_lowercase();
    let mut best: Option<(f64, usize)> = None;
    let mut first_alive: Option<usize> = None;
    for (i, n) in nodes.iter().enumerate() {
        if !n.is_alive {
            continue;
        }
        if first_alive.is_none() {
            first_alive = Some(i);
        }
        let mut score = 0.0f64;
        for spec in &n.specialties {
            if task_lower.contains(&spec.to_lowercase()) {
                score += 10.0;
            }
        }
        for skill in &n.skills {
            if task_lower.contains(&skill.to_lowercase()) {
                score += 5.0;
            }
        }
        if task_lower.contains(&n.role.to_lowercase()) {
            score += 8.0;
        }
        if let Some(rs) = required_specialty {
            if n.specialties.iter().any(|s| s == rs) {
                score += 20.0;
            } else {
                continue;
            }
        }
        score -= n.task_count * 0.1;
        if score > 0.0 {
            let replace = match best {
                Some((bs, _)) => score > bs,
                None => true,
            };
            if replace {
                best = Some((score, i));
            }
        }
    }
    match best {
        Some((score, i)) => RoutingResult {
            node_id: Some(nodes[i].node_id.clone()),
            score,
            needs_semantic: score < 10.0,
        },
        None => RoutingResult {
            node_id: first_alive.map(|i| nodes[i].node_id.clone()),
            score: 0.0,
            needs_semantic: false,
        },
    }
}

/// `reconfigure_mesh`: redistribute the orphan specialties of failed nodes round-robin onto the
/// alive nodes (skipping a specialty already present), one topology-version bump when anything
/// was removed. Mutates the alive nodes' specialties exactly as the oracle does.
#[derive(Debug, Clone)]
pub struct ReconfigureOutcome {
    pub action: &'static str,
    pub alive: usize,
    pub failed: usize,
    pub specialties_redistributed: usize,
    pub topology_version_delta: usize,
}

pub fn reconfigure_mesh(nodes: &mut [MeshNode]) -> ReconfigureOutcome {
    let alive_idx: Vec<usize> = (0..nodes.len()).filter(|&i| nodes[i].is_alive).collect();
    let failed_idx: Vec<usize> = (0..nodes.len()).filter(|&i| !nodes[i].is_alive).collect();
    if failed_idx.is_empty() {
        return ReconfigureOutcome {
            action: "none",
            alive: alive_idx.len(),
            failed: 0,
            specialties_redistributed: 0,
            topology_version_delta: 0,
        };
    }
    let mut redistributed = 0usize;
    for &di in &failed_idx {
        let orphan: Vec<String> = nodes[di].specialties.clone();
        if !orphan.is_empty() && !alive_idx.is_empty() {
            for (k, spec) in orphan.iter().enumerate() {
                let ti = alive_idx[k % alive_idx.len()];
                if !nodes[ti].specialties.contains(spec) {
                    nodes[ti].specialties.push(spec.clone());
                    redistributed += 1;
                }
            }
        }
    }
    ReconfigureOutcome {
        action: "reconfigured",
        alive: alive_idx.len(),
        failed: failed_idx.len(),
        specialties_redistributed: redistributed,
        topology_version_delta: 1,
    }
}

/// `get_stats`: node counts, total routed tasks (the task_count sum), and the connection count
/// from the same three heuristics `compute_connections` applies.
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub total_nodes: usize,
    pub alive_nodes: usize,
    pub total_tasks_routed: f64,
    pub connections: usize,
}

pub fn get_stats(nodes: &[MeshNode]) -> MeshStats {
    let alive = nodes.iter().filter(|n| n.is_alive).count();
    let total_tasks: f64 = nodes.iter().map(|n| n.task_count).sum();
    MeshStats {
        total_nodes: nodes.len(),
        alive_nodes: alive,
        total_tasks_routed: total_tasks,
        connections: compute_connections(nodes).len(),
    }
}

// ── MeshNodeSchema validators (swarm_v2/core/agent_mesh.py:157-192) ────────────────────────────
// The three pure predicates the oracle's Pydantic schema applies. Each returns ACCEPTED vs not:
// the oracle returns the value on success, raises (heartbeat/host) or `logger.warning`s (node).
// The clock arrives as an argument because firmware has no wall clock in this path.

/// `validate_heartbeat_timestamp`: not more than 300 s in the future, not before year 2000.
pub fn heartbeat_timestamp_valid(v: f64, now: f64) -> bool {
    !(v > now + 300.0) && !(v < 946684800.0)
}

/// `validate_host_format`: "localhost" or a dotted quad of 4 octets each 0..=255.
/// The oracle's `int(part)` strips surrounding whitespace and accepts a leading +/-; `trim().parse`
/// mirrors that for realistic octets. Pathological `int` literals (hex/underscore) diverge and are
/// out of scope for an IP field.
pub fn host_format_valid(v: &str) -> bool {
    if v == "localhost" {
        return true;
    }
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        match part.trim().parse::<i64>() {
            Ok(n) if (0..=255).contains(&n) => {}
            _ => return false,
        }
    }
    true
}

/// `validate_node_consistency`: whether `node_id` matches the name-derived id (lowercased,
/// spaces→underscores, first 64 chars) or at least shares its 32-char prefix. Empty fields
/// skip the check (the oracle's `if self.node_id and self.name`).
pub fn node_consistency_ok(node_id: &str, name: &str) -> bool {
    if node_id.is_empty() || name.is_empty() {
        return true;
    }
    let expected: String = name
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .take(64)
        .collect();
    let prefix_len = expected.chars().count().min(32);
    let prefix: String = expected.chars().take(prefix_len).collect();
    node_id == expected || node_id.starts_with(&prefix)
}

impl MeshNodeSchema {
    pub fn new() -> Self {
        MeshNodeSchema {
            node_id: String::new(),
            name: String::new(),
            role: String::new(),
            host: String::new(),
            port: 0,
            specialties: vec![],
            skills: vec![],
            hardware: vec![],
            status: String::new(),
            last_heartbeat: 0.0,
            joined_at: String::new(),
            task_count: 0,
            peer_connections: 0,
        }
    }
}

//  get_agent_mesh() 
static SINGLETON_AGENT_MESH: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_agent_mesh() -> u64 {
    // Singleton handle (lazy-init pattern from agent_mesh.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_AGENT_MESH.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_AGENT_MESH.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
