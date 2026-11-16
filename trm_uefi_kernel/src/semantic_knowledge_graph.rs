//! TRM Swarm OS — semantic_knowledge_graph (no_std port).
//! Semantic Knowledge Graph (SKG): dynamic self-updating graph of Swarm OS
//! topology. 1:1 deterministic core from swarm_v2/core/semantic_knowledge_graph.py:
//!   - bootstrap graph: 7 core nodes (archi, devo, logic, shield, swarm_api,
//!     telemetry_fabric, pbft_consensus) + 4 seeded edges (e1..e4);
//!   - add_node / add_edge with adjacency bookkeeping;
//!   - ingest_telemetry_event: node health update + anomaly log when
//!     health_score < 0.6;
//!   - get_topology_snapshot (counts) and infer_dependencies (adjacency).
//! The pydantic BaseModel validation, time.time() wall clocks and the
//! Dict[str,Any] payloads are represented by the fields below; payload keys
//! are stored as (key, value) pairs preserving update() merge semantics.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// One node of the Semantic Knowledge Graph (GraphNode, deterministic subset).
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub node_id: String,
    pub node_type: String, // "service" | "agent" | "resource" | "artifact" | "security_policy"
    pub label: String,
    pub health_score: f64,
    /// properties Dict[str, Any] as (key, value) pairs (numeric subset).
    pub properties: Vec<(String, f64)>,
}

impl GraphNode {
    pub fn new(node_id: &str, node_type: &str, label: &str, health_score: f64) -> Self {
        GraphNode {
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            label: label.to_string(),
            health_score,
            properties: vec![],
        }
    }
}

/// One edge of the SKG (GraphEdge, deterministic subset).
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String, // "depends_on" | "routes_to" | "monitors" | "executes" | "secures"
    pub weight: f64,           // python default 1.0
}

impl GraphEdge {
    pub fn new(edge_id: &str, source_id: &str, target_id: &str, relation_type: &str) -> Self {
        GraphEdge {
            edge_id: edge_id.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation_type: relation_type.to_string(),
            weight: 1.0,
        }
    }
}

/// A logged anomaly: node_id whose health dropped below 0.6.
#[derive(Debug, Clone)]
pub struct AnomalyRecord {
    pub node_id: String,
    pub health_score: f64,
    /// event.get("reason", "Degraded performance detected") — python default.
    pub reason: String,
}

/// SemanticKnowledgeGraph — dynamic topology engine (python class).
#[derive(Debug, Clone)]
pub struct SemanticKnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// adjacency: node_id -> ordered target list (python Dict[str, List[str]]).
    pub adjacency: Vec<(String, Vec<String>)>,
    pub anomaly_log: Vec<AnomalyRecord>,
}

impl SemanticKnowledgeGraph {
    /// __init__: empty stores then `_initialize_bootstrap_graph()`.
    pub fn new() -> Self {
        let mut g = SemanticKnowledgeGraph {
            nodes: vec![],
            edges: vec![],
            adjacency: vec![],
            anomaly_log: vec![],
        };
        g.initialize_bootstrap_graph();
        g
    }

    /// _initialize_bootstrap_graph(): seed 7 core Swarm OS nodes + 4 edges.
    pub fn initialize_bootstrap_graph(&mut self) {
        // core_nodes list (1:1 python). Properties dicts are numeric subset only;
        // string-valued properties (role/model/status) are carried in the label.
        self.add_node(GraphNode::new("archi", "agent", "Architect (Archi)", 1.0));
        self.add_node(GraphNode::new("devo", "agent", "Lead Developer (Devo)", 1.0));
        self.add_node(GraphNode::new("logic", "agent", "Reasoning Engine (Logic)", 1.0));
        self.add_node(GraphNode::new("shield", "agent", "Security Auditor (Shield)", 1.0));
        self.add_node(GraphNode::new("swarm_api", "service", "Swarm API Server", 1.0));
        self.add_node(GraphNode::new("telemetry_fabric", "resource", "Telemetry Fabric", 1.0));
        self.add_node(GraphNode::new("pbft_consensus", "service", "PBFT Consensus Engine", 1.0));

        // Seed initial edges.
        self.add_edge(GraphEdge::new("e1", "archi", "devo", "routes_to"));
        self.add_edge(GraphEdge::new("e2", "archi", "logic", "routes_to"));
        self.add_edge(GraphEdge::new("e3", "shield", "swarm_api", "secures"));
        self.add_edge(GraphEdge::new("e4", "telemetry_fabric", "archi", "monitors"));
    }

    /// add_node: `self.nodes[node.node_id] = node`; ensure adjacency entry.
    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.node_id.clone();
        // dict assignment overwrites a previous node with the same id.
        if let Some(slot) = self.nodes.iter_mut().find(|n| n.node_id == id) {
            *slot = node;
        } else {
            self.nodes.push(node);
        }
        if !self.adjacency.iter().any(|(k, _)| *k == id) {
            self.adjacency.push((id, vec![]));
        }
    }

    /// add_edge: store edge; append target to source adjacency if not present.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        let src = edge.source_id.clone();
        let tgt = edge.target_id.clone();
        // dict assignment by edge_id: overwrite duplicates.
        if let Some(slot) = self.edges.iter_mut().find(|e| e.edge_id == edge.edge_id) {
            *slot = edge;
        } else {
            self.edges.push(edge);
        }
        // python: only when edge.source_id is already an adjacency key
        if let Some((_, targets)) = self.adjacency.iter_mut().find(|(k, _)| *k == src) {
            if !targets.iter().any(|t| *t == tgt) {
                targets.push(tgt);
            }
        }
    }

    fn find_node_mut(&mut self, node_id: &str) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|n| n.node_id == node_id)
    }

    pub fn node_health(&self, node_id: &str) -> Option<f64> {
        self.nodes.iter().find(|n| n.node_id == node_id).map(|n| n.health_score)
    }

    /// Update (merge) a property key on a node (dict .update semantics).
    pub fn set_property(&mut self, node_id: &str, key: &str, value: f64) {
        if let Some(node) = self.find_node_mut(node_id) {
            if let Some((_, v)) = node.properties.iter_mut().find(|(k, _)| k == key) {
                *v = value;
            } else {
                node.properties.push((key.to_string(), value));
            }
        }
    }

    /// ingest_telemetry_event(event) — deterministic core.
    /// `node_id` is the resolved event node id (see `event_node_id`),
    /// `health` is event.get("health_score"), `payload` merges properties.
    /// Anomaly is logged when the (defaulted) health < 0.6 regardless of
    /// whether the node is known.
    pub fn ingest_telemetry_event(
        &mut self,
        node_id: &str,
        health: Option<f64>,
        payload: &[(&str, f64)],
    ) {
        if let Some(node) = self.find_node_mut(node_id) {
            if let Some(h) = health {
                node.health_score = h;
            }
            for (k, v) in payload {
                if let Some((_, vv)) = node.properties.iter_mut().find(|(kk, _)| kk == k) {
                    *vv = *v;
                } else {
                    node.properties.push(((*k).to_string(), *v));
                }
            }
        }

        // health = event.get("health_score", 1.0)
        let h = health.unwrap_or(1.0);
        if h < 0.6 {
            self.anomaly_log.push(AnomalyRecord {
                node_id: node_id.to_string(),
                health_score: h,
                reason: "Degraded performance detected".to_string(),
            });
        }
    }

    /// Resolve `event.get("node_id") or event.get("source") or "swarm_api"`.
    /// Empty/None values are falsy, mirroring python `or` chaining.
    pub fn event_node_id(node: Option<&str>, source: Option<&str>) -> String {
        let n = node.filter(|s| !s.is_empty());
        let s = source.filter(|s| !s.is_empty());
        n.or(s).unwrap_or("swarm_api").to_string()
    }

    /// get_topology_snapshot — counts of the live graph (dict subset).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Python snapshot exposes only the last 10 anomalies.
    pub fn anomaly_count(&self) -> usize {
        self.anomaly_log.len().min(10)
    }

    pub fn anomaly_log_len(&self) -> usize {
        self.anomaly_log.len()
    }

    /// infer_dependencies(node_id) — adjacency list copy (python .get default []).
    pub fn infer_dependencies(&self, node_id: &str) -> Vec<String> {
        self.adjacency
            .iter()
            .find(|(k, _)| k == node_id)
            .map(|(_, t)| t.clone())
            .unwrap_or_default()
    }
}

impl Default for SemanticKnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton instance (lazy-init pattern from semantic_knowledge_graph.py).
static SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_semantic_knowledge_graph() singleton handle: 1 once initialized.
pub fn get_semantic_knowledge_graph() -> u64 {
    let v = SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Bootstrap invariants used by the host harness (7 nodes / 4 edges).
pub const BOOTSTRAP_NODE_COUNT: usize = 7;
pub const BOOTSTRAP_EDGE_COUNT: usize = 4;

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 5 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).
// imports: use alloc::string::String;


fn pnum(v: f64) -> PropertyValue {
    PropertyValue::Num(v)
}

fn ptext(s: &str) -> PropertyValue {
    PropertyValue::Text(String::from(s))
}

pub struct TopologySnapshot {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub anomalies: Vec<GraphAnomaly>,
    pub timestamp: f64,
}

pub enum PropertyValue {
    Num(f64),
    Text(String),
}

pub struct GraphAnomaly {
    pub timestamp: f64,
    pub node_id: String,
    pub health_score: f64,
    pub reason: String,
}
