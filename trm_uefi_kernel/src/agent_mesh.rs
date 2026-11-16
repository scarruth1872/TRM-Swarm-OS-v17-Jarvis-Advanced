//! TRM Swarm OS — agent_mesh (faithful no_std port).
//! P2P Agent Mesh from swarm_v2/core/agent_mesh.py:
//!   - Pydantic-v2-style field validation for MeshNode (node_id charset/length,
//!     name/role lengths, host IPv4-or-localhost with octet ranges, status
//!     vocabulary, port range, heartbeat timestamp plausibility vs `now`,
//!     specialties/skills cardinality) — ported as pure string checks (the
//!     Python uses regexes; equivalents are written out char-by-char).
//!   - AgentMesh registry, topology versioning, heartbeat propagation,
//!     stale cleanup, role/specialty queries, connection strength heuristics,
//!     find_best_node scoring, task routing, mesh reconfiguration — 1:1
//!     deterministic logic. Python reads `time.time()` everywhere; the port
//!     takes an explicit `now` parameter so every method stays deterministic.
//!   - Semantic tie-break routing uses the host `FastEmbeddingSkill` in
//!     Python; the kernel has no embedder, and Python itself catches embedder
//!     exceptions and falls back to `candidates[0]` — the kernel takes that
//!     documented exception path (// MANUAL).
//! Deep bodies: JSON state persistence (`_save_state`/`_load_state`,
//! os.makedirs at import), asyncio routing and ISO timestamps → honest
//! `// MANUAL` returns below — never invented. Wall-clock fields that
//! Python fills (joined_at, message timestamps) are dropped or caller-filled.
//! Python `round(latency, 1)` is half-even; the helper here is half-up at
//! 1 decimal (documented approximation, bit-exact except at exact .xx5).

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub const MESH_DIR: &str = "swarm_v2_mesh";
/// Node alive window: 10 minutes (Python `is_alive`, timeout default 600s).
pub const HEARTBEAT_TIMEOUT: f64 = 600.0;
/// 5-minute clock skew allowance for heartbeat timestamps.
pub const HEARTBEAT_FUTURE_SKEW: f64 = 300.0;
/// Year-2000 minimum heartbeat timestamp (Python 946684800).
pub const HEARTBEAT_MIN_TIMESTAMP: f64 = 946_684_800.0;
/// Status vocabulary (Python `^(online|offline|busy|error)$`).
pub const NODE_STATUSES: [&str; 4] = ["online", "offline", "busy", "error"];
/// Dev-role words driving connection heuristic #2.
pub const DEV_ROLE_WORDS: [&str; 4] = ["developer", "architect", "engineer", "logic"];

// ─────────────────────────────────────────────────────────────
// float helpers (no_std has no floor/round on f64)
// ─────────────────────────────────────────────────────────────

/// floor via truncation + correction (same trick as thz_wave_gateway.rs).
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < t {
        t - 1.0
    } else {
        t
    }
}

/// Round-half-up to 1 decimal (Python `round(x, 1)` is round-half-even —
/// identical except at exact .xx5 values).
pub fn round1(x: f64) -> f64 {
    floor_f64(x * 10.0 + 0.5) / 10.0
}

// ─────────────────────────────────────────────────────────────
// Field validators (Pydantic v2 MeshNodeSchema port)
// ─────────────────────────────────────────────────────────────

/// `^[a-zA-Z0-9_-]+$`, length 1..=128.
pub fn validate_node_id(node_id: &str) -> bool {
    let mut len = 0usize;
    for c in node_id.chars() {
        len += 1;
        if len > 128 {
            return false;
        }
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return false;
        }
    }
    len >= 1
}

/// Non-empty, length 1..=256.
pub fn validate_name(name: &str) -> bool {
    let len = name.chars().count();
    len >= 1 && len <= 256
}

/// Non-empty, length 1..=64.
pub fn validate_role(role: &str) -> bool {
    let len = role.chars().count();
    len >= 1 && len <= 64
}

/// `^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$|^localhost$` plus octet range check.
pub fn validate_host(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        if part.is_empty() || part.len() > 3 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        match part.parse::<u16>() {
            Ok(num) if num <= 255 => {}
            _ => return false,
        }
    }
    true
}

pub fn validate_status(status: &str) -> bool {
    NODE_STATUSES.contains(&status)
}

pub fn validate_port(port: u16) -> bool {
    // u16 already bounds 0..=65535; kept for API fidelity with pydantic ge/le.
    true
}

/// `validate_heartbeat_timestamp`: not more than 300s in the future and not
/// before year 2000 (Python compares against `time.time()` → `now` here).
pub fn validate_heartbeat(ts: f64, now: f64) -> bool {
    ts <= now + HEARTBEAT_FUTURE_SKEW && ts >= HEARTBEAT_MIN_TIMESTAMP
}

/// Cross-field sanity check (informational in Python, never enforced):
/// `node_id == expected_id || node_id.startswith(expected_id[:32])` where
/// `expected_id = name.lower().replace(' ', '_')[:64]`.
pub fn node_id_consistent(node_id: &str, name: &str) -> bool {
    let expected: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '_' } else { c })
        .take(64)
        .collect();
    let prefix: String = expected.chars().take(32).collect();
    node_id == expected || node_id.starts_with(&prefix)
}

// ─────────────────────────────────────────────────────────────
// MeshNode
// ─────────────────────────────────────────────────────────────

/// A single agent node (Python MeshNode, fields of MeshNodeSchema).
#[derive(Debug, Clone)]
pub struct MeshNode {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub host: String,
    pub port: u16,
    pub specialties: Vec<String>,
    pub skills: Vec<String>,
    pub hardware: Vec<(String, f64)>,
    pub status: String,
    pub last_heartbeat: f64,
    pub joined_at: String,
    pub task_count: u64,
    pub peer_connections: u64,
}

impl MeshNode {
    /// Construct + validate exactly like pydantic `MeshNodeSchema(**kwargs)`:
    /// node_id → name → role → host → port → list cardinalities, with
    /// `status="online"`, `last_heartbeat=now`, `task_count=0`,
    /// `peer_connections=0`.
    ///
    /// // MANUAL: `joined_at` is `datetime.now().isoformat()` in Python (no
    /// clock in the kernel) — passed in or left empty by the caller.
    pub fn validated(
        node_id: &str,
        name: &str,
        role: &str,
        host: &str,
        port: u16,
        specialties: Vec<String>,
        skills: Vec<String>,
        hardware: Vec<(String, f64)>,
        now: f64,
    ) -> Result<MeshNode, String> {
        if !validate_node_id(node_id) {
            return Err(format!(
                "Invalid node_id '{}': 1-128 chars of [a-zA-Z0-9_-]",
                node_id
            ));
        }
        if !validate_name(name) {
            return Err("name must be 1-256 characters".to_string());
        }
        if !validate_role(role) {
            return Err("role must be 1-64 characters".to_string());
        }
        if !validate_host(host) {
            return Err(format!("Invalid host format: {}", host));
        }
        validate_port(port);
        if specialties.len() > 50 {
            return Err("specialties must have at most 50 items".to_string());
        }
        if skills.len() > 50 {
            return Err("skills must have at most 50 items".to_string());
        }
        if !validate_heartbeat(now, now) {
            return Err(format!("Heartbeat timestamp {} out of valid range", now));
        }
        Ok(MeshNode {
            node_id: node_id.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            host: host.to_string(),
            port,
            specialties,
            skills,
            hardware,
            status: "online".to_string(),
            last_heartbeat: now,
            joined_at: String::new(),
            task_count: 0,
            peer_connections: 0,
        })
    }

    /// `heartbeat()`: refresh timestamp, status → online.
    pub fn heartbeat(&mut self, now: f64) {
        self.last_heartbeat = now;
        self.status = "online".to_string();
    }

    /// `is_alive`: `(time.time() - last_heartbeat) < 600`.
    pub fn is_alive(&self, now: f64) -> bool {
        (now - self.last_heartbeat) < HEARTBEAT_TIMEOUT
    }

    /// `to_dict()['latency_ms'] = round((time.time() - last_heartbeat) * 1000, 1)`.
    pub fn latency_ms(&self, now: f64) -> f64 {
        round1((now - self.last_heartbeat) * 1000.0)
    }

    /// Python `__repr__`: `MeshNode(id=..., role=..., status=...)`.
    pub fn repr_line(&self) -> String {
        format!(
            "MeshNode(id={}, role={}, status={})",
            self.node_id, self.role, self.status
        )
    }
}

// ─────────────────────────────────────────────────────────────
// Mesh log entries
// ─────────────────────────────────────────────────────────────

/// Python `self.message_log` entries carry dicts keyed by `type`; the two
/// shapes the mesh appends are kept as explicit variants (timestamps were
/// wall-clock ISO strings → // MANUAL, dropped).
#[derive(Debug, Clone)]
pub enum MeshLogEntry {
    TaskRouted {
        task: String,
        target_node: String,
        target_role: String,
        node_id: String,
    },
    MeshReconfiguration {
        failed_node: String,
        failed_node_id: String,
        orphan_specialties: Vec<String>,
        redistributed_to: Vec<String>,
    },
}

impl MeshLogEntry {
    pub fn log_type(&self) -> &'static str {
        match self {
            MeshLogEntry::TaskRouted { .. } => "task_routed",
            MeshLogEntry::MeshReconfiguration { .. } => "mesh_reconfiguration",
        }
    }
}

/// A computed logical connection (`_compute_connections`).
#[derive(Debug, Clone)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub shared_specialties: Vec<String>,
    pub strength: u64,
}

// ─────────────────────────────────────────────────────────────
// AgentMesh
// ─────────────────────────────────────────────────────────────

/// The P2P mesh fabric. `nodes` preserves insertion order (Python dict
/// semantics); lookups are linear. Topology versioning, register/unregister,
/// heartbeat, cleanup, routing, reconfiguration mirror Python 1:1.
#[derive(Debug, Clone, Default)]
pub struct AgentMesh {
    pub nodes: Vec<MeshNode>,
    pub message_log: Vec<MeshLogEntry>,
    pub topology_version: u64,
}

/// Snapshot of mesh topology (`get_topology_snapshot`, minus wall-clock and
/// per-node serialization).
#[derive(Debug, Clone)]
pub struct TopologySnapshot {
    pub version: u64,
    pub node_count: usize,
    pub alive_count: usize,
    pub connection_count: usize,
    pub message_count: usize,
}

/// `route_task` outcome.
#[derive(Debug, Clone)]
pub struct RouteResult {
    pub error: Option<String>,
    pub task: String,
    pub routed_to: Option<String>,
    pub target_role: Option<String>,
    pub node_id: Option<String>,
    pub mesh_version: u64,
}

/// `reconfigure_mesh` outcome.
#[derive(Debug, Clone)]
pub struct ReconfigOutcome {
    pub action: &'static str,
    pub alive: usize,
    pub failed: usize,
    pub specialties_redistributed: usize,
    pub topology_version: u64,
}

/// `get_stats` result.
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub total_nodes: usize,
    pub alive_nodes: usize,
    pub topology_version: u64,
    pub total_tasks_routed: u64,
    pub total_messages: usize,
    pub connections: usize,
}

impl AgentMesh {
    pub fn new() -> Self {
        // Python __init__ also restores a persisted topology_version from
        // swarm_v2_mesh/mesh_state.json → // MANUAL host IO; kernel starts 0.
        AgentMesh {
            nodes: Vec::new(),
            message_log: Vec::new(),
            topology_version: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn contains(&self, node_id: &str) -> bool {
        self.nodes.iter().any(|n| n.node_id == node_id)
    }

    /// `register_node`: new node → push + version bump (true);
    /// duplicate → heartbeat refresh only (false).
    pub fn register_node(&mut self, node: MeshNode, now: f64) -> bool {
        if self.contains(&node.node_id) {
            self.propagate_heartbeat(&node.node_id, now);
            return false;
        }
        self.nodes.push(node);
        self.topology_version += 1;
        true
    }

    /// `unregister_node`: remove + version bump (true); false when missing.
    pub fn unregister_node(&mut self, node_id: &str) -> bool {
        if let Some(idx) = self.nodes.iter().position(|n| n.node_id == node_id) {
            self.nodes.remove(idx);
            self.topology_version += 1;
            return true;
        }
        false
    }

    pub fn get_node(&self, node_id: &str) -> Option<&MeshNode> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut MeshNode> {
        self.nodes.iter_mut().find(|n| n.node_id == node_id)
    }

    /// `get_alive_nodes`.
    pub fn get_alive_nodes(&self, now: f64) -> Vec<&MeshNode> {
        self.nodes.iter().filter(|n| n.is_alive(now)).collect()
    }

    /// `get_nodes_by_role`.
    pub fn get_nodes_by_role(&self, role: &str) -> Vec<&MeshNode> {
        self.nodes.iter().filter(|n| n.role == role).collect()
    }

    /// `get_nodes_by_specialty` (exact list membership).
    pub fn get_nodes_by_specialty(&self, specialty: &str) -> Vec<&MeshNode> {
        self.nodes
            .iter()
            .filter(|n| n.specialties.iter().any(|s| s == specialty))
            .collect()
    }

    /// `propagate_heartbeat`: refresh a node's heartbeat; false if unknown.
    pub fn propagate_heartbeat(&mut self, node_id: &str, now: f64) -> bool {
        match self.get_node_mut(node_id) {
            Some(node) => {
                node.heartbeat(now);
                true
            }
            None => false,
        }
    }

    /// `cleanup_stale_nodes(timeout)`: drop nodes silent for > timeout;
    /// version bumps once when anything was removed. Returns count removed.
    pub fn cleanup_stale_nodes(&mut self, now: f64, timeout: f64) -> u64 {
        let before = self.nodes.len();
        self.nodes.retain(|n| (now - n.last_heartbeat) <= timeout);
        let removed = (before - self.nodes.len()) as u64;
        if removed > 0 {
            self.topology_version += 1;
        }
        removed
    }

    /// Insertion-ordered intersection of two specialty lists (Python uses a
    /// `set & set`; sets iterate in hash order there, Vec order here — the
    /// count feeding `strength` is identical, and this ordering is stable).
    fn shared_specialties<'a>(a: &'a [String], b: &'a [String]) -> Vec<&'a str> {
        a.iter()
            .filter(|s| b.iter().any(|t| t == *s))
            .map(|s| s.as_str())
            .collect()
    }

    /// `_compute_connections` — strength heuristics 1:1.
    pub fn compute_connections(&self) -> Vec<Connection> {
        let mut connections = Vec::new();
        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                let a = &self.nodes[i];
                let b = &self.nodes[j];
                let mut strength: u64 = 0;
                let shared = Self::shared_specialties(&a.specialties, &b.specialties);
                if !shared.is_empty() {
                    strength += (shared.len() as u64) * 2;
                }
                let a_role = a.role.to_lowercase();
                let b_role = b.role.to_lowercase();
                let a_dev = DEV_ROLE_WORDS.iter().any(|w| a_role.contains(w));
                let b_dev = DEV_ROLE_WORDS.iter().any(|w| b_role.contains(w));
                if a_dev && b_dev {
                    strength += 1;
                }
                if a.role == "Architect" || b.role == "Architect" {
                    strength += 1;
                }
                if strength > 0 {
                    connections.push(Connection {
                        from: a.node_id.clone(),
                        to: b.node_id.clone(),
                        shared_specialties: shared.iter().map(|s| s.to_string()).collect(),
                        strength,
                    });
                }
            }
        }
        connections
    }

    /// Score a candidate node for `find_best_node` (specialty +10, skill +5,
    /// role +8, required specialty +20, minus 0.1 per processed task).
    fn score_node(node: &MeshNode, task_lower: &str, required_specialty: Option<&str>) -> f64 {
        let mut score: f64 = 0.0;
        for spec in node.specialties.iter() {
            // Python: `if spec.lower() in task_lower` — one direction only.
            if task_lower.contains(&spec.to_lowercase()) {
                score += 10.0;
            }
        }
        for skill in node.skills.iter() {
            if task_lower.contains(&skill.to_lowercase()) {
                score += 5.0;
            }
        }
        if task_lower.contains(&node.role.to_lowercase()) {
            score += 8.0;
        }
        if let Some(req) = required_specialty {
            if node.specialties.iter().any(|s| s == req) {
                score += 20.0;
            } else {
                return f64::NAN; // filtered out (Python `continue`)
            }
        }
        score -= node.task_count as f64 * 0.1;
        score
    }

    /// `find_best_node(task, required_specialty)` — returns the node index.
    /// // MANUAL: when the top heuristic score is < 10 Python attempts a
    /// FastEmbeddingSkill semantic tie-break; on ANY embedder exception it
    /// logs and returns `candidates[0]`. No embedder exists in the kernel,
    /// so the documented Python exception path (top candidate) is taken.
    pub fn find_best_node(
        &self,
        task: &str,
        required_specialty: Option<&str>,
        now: f64,
    ) -> Option<usize> {
        let task_lower = task.to_lowercase();
        let mut candidates: Vec<(f64, usize)> = Vec::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            if !node.is_alive(now) {
                continue;
            }
            let score = Self::score_node(node, &task_lower, required_specialty);
            if score.is_nan() {
                continue;
            }
            if score > 0.0 {
                candidates.push((score, idx));
            }
        }
        if candidates.is_empty() {
            // Fallback: first alive node (Python `alive[0]`).
            return self
                .nodes
                .iter()
                .position(|n| n.is_alive(now));
        }
        // Python `sorted(key=..., reverse=True)` is stable → ties keep node
        // insertion order, exactly like Rust's stable sort with reversed keys.
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        Some(candidates[0].1)
    }

    /// `route_task(task, target_node_id, required_specialty)`.
    pub fn route_task(
        &mut self,
        task: &str,
        target_node_id: Option<&str>,
        required_specialty: Option<&str>,
        now: f64,
    ) -> RouteResult {
        let chosen: Option<usize> = if let Some(tid) = target_node_id {
            if self.contains(tid) {
                self.nodes.iter().position(|n| n.node_id == tid)
            } else {
                self.find_best_node(task, required_specialty, now)
            }
        } else {
            self.find_best_node(task, required_specialty, now)
        };

        let Some(idx) = chosen else {
            return RouteResult {
                error: Some("No suitable node found on the mesh".to_string()),
                task: task.to_string(),
                routed_to: None,
                target_role: None,
                node_id: None,
                mesh_version: self.topology_version,
            };
        };

        let node = &mut self.nodes[idx];
        node.task_count += 1;
        let (node_name, node_role, node_id) =
            (node.name.clone(), node.role.clone(), node.node_id.clone());

        // Python: `self._save_state()` → // MANUAL host IO.
        // Python: task[:80] truncation for the log entry.
        let task_trunc: String = task.chars().take(80).collect();
        self.message_log.push(MeshLogEntry::TaskRouted {
            task: task_trunc,
            target_node: node_name.clone(),
            target_role: node_role.clone(),
            node_id: node_id.clone(),
        });

        RouteResult {
            error: None,
            task: task.to_string(),
            routed_to: Some(node_name),
            target_role: Some(node_role),
            node_id: Some(node_id),
            mesh_version: self.topology_version,
        }
    }

    /// `get_topology_snapshot` (alias `get_topology`) — deterministic counts.
    /// // MANUAL: Python embeds per-node `to_dict()` blobs and an ISO
    /// timestamp; node details are reachable via `get_node`/queries instead.
    pub fn get_topology(&self, now: f64) -> TopologySnapshot {
        TopologySnapshot {
            version: self.topology_version,
            node_count: self.nodes.len(),
            alive_count: self.get_alive_nodes(now).len(),
            connection_count: self.compute_connections().len(),
            message_count: self.message_log.len(),
        }
    }

    pub fn get_topology_snapshot(&self, now: f64) -> TopologySnapshot {
        self.get_topology(now)
    }

    /// `get_stats`.
    pub fn get_stats(&self, now: f64) -> MeshStats {
        let alive = self.get_alive_nodes(now).len();
        MeshStats {
            total_nodes: self.nodes.len(),
            alive_nodes: alive,
            topology_version: self.topology_version,
            total_tasks_routed: self.nodes.iter().map(|n| n.task_count).sum(),
            total_messages: self.message_log.len(),
            connections: self.compute_connections().len(),
        }
    }

    /// `get_message_log(limit)`: last `limit` entries (Python `[-limit:]`).
    pub fn get_message_log(&self, limit: usize) -> Vec<&MeshLogEntry> {
        let total = self.message_log.len();
        let start = total.saturating_sub(limit);
        self.message_log[start..].iter().collect()
    }

    /// `reconfigure_mesh`: detect dead nodes, round-robin redistribute their
    /// orphan specialties onto alive nodes, mark them offline (kept in the
    /// registry for recovery), version bump. Returns the outcome counts.
    pub fn reconfigure_mesh(&mut self, now: f64) -> ReconfigOutcome {
        let alive_idx: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.is_alive(now))
            .map(|(i, _)| i)
            .collect();
        let failed_idx: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.is_alive(now))
            .map(|(i, _)| i)
            .collect();

        if failed_idx.is_empty() {
            return ReconfigOutcome {
                action: "none",
                alive: alive_idx.len(),
                failed: 0,
                specialties_redistributed: 0,
                topology_version: self.topology_version,
            };
        }

        let mut redistributed: usize = 0;
        for dead in &failed_idx {
            let orphan_specialties: Vec<String> =
                self.nodes[*dead].specialties.clone();
            if !orphan_specialties.is_empty() && !alive_idx.is_empty() {
                for (i, spec) in orphan_specialties.iter().enumerate() {
                    let target = alive_idx[i % alive_idx.len()];
                    let t = &mut self.nodes[target];
                    if !t.specialties.iter().any(|s| s == spec) {
                        t.specialties.push(spec.clone());
                        redistributed += 1;
                    }
                }
            }
            // Python logs redistribution targets as the alive nodes' names.
            let redistributed_to: Vec<String> =
                alive_idx.iter().map(|i| self.nodes[*i].name.clone()).collect();
            self.message_log.push(MeshLogEntry::MeshReconfiguration {
                failed_node: self.nodes[*dead].name.clone(),
                failed_node_id: self.nodes[*dead].node_id.clone(),
                orphan_specialties: orphan_specialties.clone(),
                redistributed_to,
            });
            self.nodes[*dead].status = "offline".to_string();
        }

        self.topology_version += 1;
        ReconfigOutcome {
            action: "reconfigured",
            alive: alive_idx.len(),
            failed: failed_idx.len(),
            specialties_redistributed: redistributed,
            topology_version: self.topology_version,
        }
    }
}

/// Python module singleton `get_agent_mesh()` lazily builds ONE shared
/// AgentMesh. No_std kernels avoid global mutable state, so this returns a
/// fresh mesh per call (the Python registry itself is re-populated from disk
/// on every startup anyway; persistence is // MANUAL). Kernel callers own
/// their mesh instance.
pub fn get_agent_mesh() -> AgentMesh {
    AgentMesh::new()
}

// ── grafted from the v20 base tree (G188) ────────────────────────────────────────────────────────
// Six free functions that exist only on the base side. Unlike radix_trie_matcher's methods, these take
// primitives and slices and touch no struct, so they graft verbatim with no adaptation and no risk of
// carrying a foreign data model into this file.

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

pub fn heartbeat_timestamp_valid(v: f64, now: f64) -> bool {
    !(v > now + 300.0) && !(v < 946684800.0)
}

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
