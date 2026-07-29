"""
Semantic Knowledge Graph (SKG) Module — Phase 1: Cognitive Foundation
Maintains a dynamic, self-updating graph of Swarm OS topology, component interdependencies,
historical performance metrics, security events, and environmental context.
"""

import logging
import time
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("SemanticKnowledgeGraph")


class GraphNode(BaseModel):
    """Represents an entity in the Semantic Knowledge Graph."""
    node_id: str
    node_type: str  # "service", "agent", "resource", "artifact", "security_policy"
    label: str
    properties: Dict[str, Any] = Field(default_factory=dict)
    health_score: float = 1.0
    created_at: float = Field(default_factory=time.time)
    updated_at: float = Field(default_factory=time.time)


class GraphEdge(BaseModel):
    """Represents a relationship between two nodes in the SKG."""
    edge_id: str
    source_id: str
    target_id: str
    relation_type: str  # "depends_on", "routes_to", "monitors", "executes", "secures"
    weight: float = 1.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SemanticKnowledgeGraph:
    """
    Dynamic Knowledge Graph engine representing the real-time topology,
    dependencies, and operational state of the Swarm OS ecosystem.
    """

    def __init__(self):
        self.nodes: Dict[str, GraphNode] = {}
        self.edges: Dict[str, GraphEdge] = {}
        self.adjacency: Dict[str, List[str]] = {}
        self.anomaly_log: List[Dict[str, Any]] = []
        self._initialize_bootstrap_graph()

    def _initialize_bootstrap_graph(self):
        """Seed the graph with initial Swarm OS core architecture nodes."""
        core_nodes = [
            GraphNode(node_id="archi", node_type="agent", label="Architect (Archi)", properties={"role": "Architect", "model": "gemini"}),
            GraphNode(node_id="devo", node_type="agent", label="Lead Developer (Devo)", properties={"role": "Lead Developer"}),
            GraphNode(node_id="logic", node_type="agent", label="Reasoning Engine (Logic)", properties={"role": "Reasoning Engine"}),
            GraphNode(node_id="shield", node_type="agent", label="Security Auditor (Shield)", properties={"role": "Security Auditor"}),
            GraphNode(node_id="swarm_api", node_type="service", label="Swarm API Server", properties={"port": 8021}),
            GraphNode(node_id="telemetry_fabric", node_type="resource", label="Telemetry Fabric", properties={"status": "active"}),
            GraphNode(node_id="pbft_consensus", node_type="service", label="PBFT Consensus Engine", properties={"f_faults": 1}),
        ]
        for node in core_nodes:
            self.add_node(node)

        # Seed initial edges
        self.add_edge(GraphEdge(edge_id="e1", source_id="archi", target_id="devo", relation_type="routes_to"))
        self.add_edge(GraphEdge(edge_id="e2", source_id="archi", target_id="logic", relation_type="routes_to"))
        self.add_edge(GraphEdge(edge_id="e3", source_id="shield", target_id="swarm_api", relation_type="secures"))
        self.add_edge(GraphEdge(edge_id="e4", source_id="telemetry_fabric", target_id="archi", relation_type="monitors"))

    def add_node(self, node: GraphNode):
        """Add or update a node in the SKG."""
        self.nodes[node.node_id] = node
        if node.node_id not in self.adjacency:
            self.adjacency[node.node_id] = []
        logger.debug(f"[SKG] Added node: {node.node_id} ({node.node_type})")

    def add_edge(self, edge: GraphEdge):
        """Add an edge establishing interdependency between nodes."""
        self.edges[edge.edge_id] = edge
        if edge.source_id in self.adjacency:
            if edge.target_id not in self.adjacency[edge.source_id]:
                self.adjacency[edge.source_id].append(edge.target_id)
        logger.debug(f"[SKG] Added edge: {edge.source_id} -[{edge.relation_type}]-> {edge.target_id}")

    def ingest_telemetry_event(self, event: Dict[str, Any]):
        """Ingest raw telemetry event and dynamically update node properties and health scores."""
        node_id = event.get("node_id") or event.get("source") or "swarm_api"
        if node_id in self.nodes:
            node = self.nodes[node_id]
            node.updated_at = time.time()
            if "health_score" in event:
                node.health_score = float(event["health_score"])
            if "properties" in event:
                node.properties.update(event["properties"])

        # Anomaly detection: if health drops below 0.6, log anomaly
        health = event.get("health_score", 1.0)
        if health < 0.6:
            anomaly = {
                "timestamp": time.time(),
                "node_id": node_id,
                "health_score": health,
                "reason": event.get("reason", "Degraded performance detected"),
            }
            self.anomaly_log.append(anomaly)
            logger.warning(f"[SKG Anomaly] Node {node_id} degraded to {health}")

    def get_topology_snapshot(self) -> Dict[str, Any]:
        """Return full snapshot of the Semantic Knowledge Graph."""
        return {
            "node_count": len(self.nodes),
            "edge_count": len(self.edges),
            "nodes": [n.model_dump() for n in self.nodes.values()],
            "edges": [e.model_dump() for e in self.edges.values()],
            "anomalies": self.anomaly_log[-10:],
            "timestamp": time.time(),
        }

    def infer_dependencies(self, node_id: str) -> List[str]:
        """Traverse the graph to return all dependent target nodes for a given source node."""
        return self.adjacency.get(node_id, [])


# Singleton instance
_skg_instance: Optional[SemanticKnowledgeGraph] = None


def get_semantic_knowledge_graph() -> SemanticKnowledgeGraph:
    global _skg_instance
    if _skg_instance is None:
        _skg_instance = SemanticKnowledgeGraph()
    return _skg_instance
