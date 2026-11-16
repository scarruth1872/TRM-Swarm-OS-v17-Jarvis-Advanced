//! TRM Swarm OS - semantic_knowledge_graph (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/semantic_knowledge_graph.py.
//! Functions marked // MANUAL need human verification of math semantics.
//!
//! Ported here (oracle names kept; `_initialize_bootstrap_graph` is spelled without the
//! leading underscore, which name-based audits normalise away):
//!   GraphNode / GraphEdge data model, SemanticKnowledgeGraph (bootstrap graph, add_node,
//!   add_edge with adjacency dedupe, ingest_telemetry_event + the health < 0.6 anomaly
//!   rule, get_topology_snapshot with the last-10 anomaly window, infer_dependencies).
//! Host-side remainder, deliberately NOT ported: `logging.getLogger` debug/warning lines,
//! wall-clock stamps (`time.time()`; a no_std module has no clock, so the caller passes
//! `now: f64`), pydantic validation / `model_dump()`, and the ctypes CDYLIB loader that
//! wraps the oracle file. The pydantic field defaults are reproduced (health_score 1.0,
//! weight 1.0) because generated zeroed defaults made a fresh node read as unhealthy.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

//  Property values
//  The oracle stores Dict[str, Any] in GraphNode.properties / GraphEdge.metadata, and its
//  own bootstrap payloads are strings ("Architect", "gemini", "active") beside ints
//  (port 8021, f_faults 1). A (String, f64) pair cannot carry those, so the value type is
//  widened to the oracle's two shapes; numeric payloads stay f64.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Num(f64),
    Text(String),
}


//  GraphNode 
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub node_id: String,
    pub node_type: String,
    pub label: String,
    pub properties: Vec<(String, PropertyValue)>,
    pub health_score: f64,
    pub created_at: f64,
    pub updated_at: f64,
}

impl GraphNode {
    pub fn new() -> Self {
        GraphNode {
            node_id: String::new(),
            node_type: String::new(),
            label: String::new(),
            properties: vec![],
            health_score: 1.0, // oracle GraphNode default (pydantic), not 0.0
            created_at: 0.0,
            updated_at: 0.0,
        }
    }
}

//  GraphEdge 
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub metadata: Vec<(String, PropertyValue)>,
}

impl GraphEdge {
    pub fn new() -> Self {
        GraphEdge {
            edge_id: String::new(),
            source_id: String::new(),
            target_id: String::new(),
            relation_type: String::new(),
            weight: 1.0, // oracle GraphEdge default (pydantic), not 0.0
            metadata: vec![],
        }
    }
}

//  get_semantic_knowledge_graph() 
static SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_semantic_knowledge_graph() -> u64 {
    // Singleton handle (lazy-init pattern from semantic_knowledge_graph.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SEMANTIC_KNOWLEDGE_GRAPH.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}


//  ---------------------------------------------------------------------------
//  Property-value constructors (glue for the oracle's dict literals; no oracle
//  counterpart, they only name the two PropertyValue shapes).
fn ptext(s: &str) -> PropertyValue {
    PropertyValue::Text(String::from(s))
}

fn pnum(v: f64) -> PropertyValue {
    PropertyValue::Num(v)
}


//  One entry of the oracle's anomaly_log: a plain Dict[str, Any] there.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphAnomaly {
    pub timestamp: f64,
    pub node_id: String,
    pub health_score: f64,
    pub reason: String,
}

//  Incoming telemetry event (oracle: a Dict[str, Any] argument). Option models the
//  oracle's `"key" in event` / `event.get(key)` distinction; empty strings are kept
//  apart from absence because the oracle's `or` chain treats "" as falsy.
#[derive(Debug, Clone, Default)]
pub struct TelemetryEvent {
    pub node_id: Option<String>,
    pub source: Option<String>,
    pub health_score: Option<f64>,
    pub reason: Option<String>,
    pub properties: Vec<(String, PropertyValue)>,
}

//  get_topology_snapshot() result (oracle: a Dict[str, Any] of counts + model_dump lists).
#[derive(Debug, Clone)]
pub struct TopologySnapshot {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub anomalies: Vec<GraphAnomaly>,
    pub timestamp: f64,
}


//  SemanticKnowledgeGraph
//  The oracle's dicts are insertion-ordered, so the maps are Vec-backed and keep the
//  oracle's ordering (Python replaces a value in place and keeps the key's position).
#[derive(Debug, Clone)]
pub struct SemanticKnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub adjacency: Vec<(String, Vec<String>)>,
    pub anomaly_log: Vec<GraphAnomaly>,
}

impl SemanticKnowledgeGraph {
    /// `now` is the host clock the oracle reads from `time.time()`; a no_std module has
    /// no clock, so every timestamp in the graph is the caller-supplied value.
    pub fn new(now: f64) -> Self {
        let mut graph = SemanticKnowledgeGraph {
            nodes: vec![],
            edges: vec![],
            adjacency: vec![],
            anomaly_log: vec![],
        };
        graph.initialize_bootstrap_graph(now);
        graph
    }

    /// oracle: `_initialize_bootstrap_graph` (the leading underscore is stripped here --
    /// name-based audits normalise it away). 7 core nodes + 4 seeded edges, values 1:1.
    pub fn initialize_bootstrap_graph(&mut self, now: f64) {
        let core_nodes: Vec<GraphNode> = vec![
            GraphNode {
                node_id: String::from("archi"),
                node_type: String::from("agent"),
                label: String::from("Architect (Archi)"),
                properties: vec![
                    (String::from("role"), ptext("Architect")),
                    (String::from("model"), ptext("gemini")),
                ],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("devo"),
                node_type: String::from("agent"),
                label: String::from("Lead Developer (Devo)"),
                properties: vec![(String::from("role"), ptext("Lead Developer"))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("logic"),
                node_type: String::from("agent"),
                label: String::from("Reasoning Engine (Logic)"),
                properties: vec![(String::from("role"), ptext("Reasoning Engine"))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("shield"),
                node_type: String::from("agent"),
                label: String::from("Security Auditor (Shield)"),
                properties: vec![(String::from("role"), ptext("Security Auditor"))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("swarm_api"),
                node_type: String::from("service"),
                label: String::from("Swarm API Server"),
                properties: vec![(String::from("port"), pnum(8021.0))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("telemetry_fabric"),
                node_type: String::from("resource"),
                label: String::from("Telemetry Fabric"),
                properties: vec![(String::from("status"), ptext("active"))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
            GraphNode {
                node_id: String::from("pbft_consensus"),
                node_type: String::from("service"),
                label: String::from("PBFT Consensus Engine"),
                properties: vec![(String::from("f_faults"), pnum(1.0))],
                health_score: 1.0,
                created_at: now,
                updated_at: now,
            },
        ];
        for node in core_nodes {
            self.add_node(node);
        }

        self.add_edge(GraphEdge {
            edge_id: String::from("e1"),
            source_id: String::from("archi"),
            target_id: String::from("devo"),
            relation_type: String::from("routes_to"),
            weight: 1.0,
            metadata: vec![],
        });
        self.add_edge(GraphEdge {
            edge_id: String::from("e2"),
            source_id: String::from("archi"),
            target_id: String::from("logic"),
            relation_type: String::from("routes_to"),
            weight: 1.0,
            metadata: vec![],
        });
        self.add_edge(GraphEdge {
            edge_id: String::from("e3"),
            source_id: String::from("shield"),
            target_id: String::from("swarm_api"),
            relation_type: String::from("secures"),
            weight: 1.0,
            metadata: vec![],
        });
        self.add_edge(GraphEdge {
            edge_id: String::from("e4"),
            source_id: String::from("telemetry_fabric"),
            target_id: String::from("archi"),
            relation_type: String::from("monitors"),
            weight: 1.0,
            metadata: vec![],
        });
    }

    /// oracle: `add_node` -- dict assignment (existing id is replaced in place, keeping its
    /// position) plus an adjacency entry created only when the id has none.
    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.node_id.clone();
        match self.nodes.iter().position(|n| n.node_id == id) {
            Some(i) => {
                self.nodes[i] = node;
            }
            None => {
                self.nodes.push(node);
            }
        }
        if !self.adjacency.iter().any(|(k, _)| *k == id) {
            self.adjacency.push((id, vec![]));
        }
    }

    /// oracle: `add_edge` -- the edge is stored even when its source is unknown, and the
    /// adjacency entry grows only if source exists and the target is not already listed.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        let edge_id = edge.edge_id.clone();
        let source_id = edge.source_id.clone();
        let target_id = edge.target_id.clone();
        match self.edges.iter().position(|e| e.edge_id == edge_id) {
            Some(i) => {
                self.edges[i] = edge;
            }
            None => {
                self.edges.push(edge);
            }
        }
        if let Some(entry) = self.adjacency.iter_mut().find(|(k, _)| *k == source_id) {
            let targets = &mut entry.1;
            if !targets.iter().any(|t| *t == target_id) {
                targets.push(target_id);
            }
        }
    }

    /// oracle: `ingest_telemetry_event` -- resolves the node id through
    /// `event.get("node_id") or event.get("source") or "swarm_api"` (empty strings are
    /// falsy and fall through), updates updated_at / health_score / properties when the
    /// node exists, then logs an anomaly while `health < 0.6` (strict: 0.6 is healthy).
    /// The anomaly keeps the RAW health value and the same `now` stamp as updated_at.
    pub fn ingest_telemetry_event(&mut self, event: &TelemetryEvent, now: f64) {
        let node_id: String = event
            .node_id
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| event.source.clone().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| String::from("swarm_api"));

        if let Some(i) = self.nodes.iter().position(|n| n.node_id == node_id) {
            self.nodes[i].updated_at = now;
            if let Some(health) = event.health_score {
                self.nodes[i].health_score = health;
            }
            if !event.properties.is_empty() {
                // dict.update(): an existing key keeps its position, a new key is appended.
                for (key, value) in event.properties.iter() {
                    match self.nodes[i].properties.iter().position(|(k, _)| k == key) {
                        Some(j) => {
                            self.nodes[i].properties[j] = (key.clone(), value.clone());
                        }
                        None => {
                            self.nodes[i].properties.push((key.clone(), value.clone()));
                        }
                    }
                }
            }
        }

        // Anomaly detection: health < 0.6 (absent health_score defaults to 1.0, no anomaly).
        let health = event.health_score.unwrap_or(1.0);
        if health < 0.6 {
            self.anomaly_log.push(GraphAnomaly {
                timestamp: now,
                node_id,
                health_score: health,
                reason: event
                    .reason
                    .clone()
                    .unwrap_or_else(|| String::from("Degraded performance detected")),
            });
        }
    }

    /// oracle: `get_topology_snapshot` -- counts, full node/edge lists, the LAST 10
    /// anomalies (`anomaly_log[-10:]`, original order) and the caller's clock.
    pub fn get_topology_snapshot(&self, now: f64) -> TopologySnapshot {
        let start = self.anomaly_log.len().saturating_sub(10);
        TopologySnapshot {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            anomalies: self.anomaly_log[start..].to_vec(),
            timestamp: now,
        }
    }

    /// oracle: `infer_dependencies` -- `adjacency.get(node_id, [])` (unknown id -> empty).
    pub fn infer_dependencies(&self, node_id: &str) -> Vec<String> {
        match self.adjacency.iter().find(|(k, _)| k.as_str() == node_id) {
            Some((_, targets)) => targets.clone(),
            None => vec![],
        }
    }
}
