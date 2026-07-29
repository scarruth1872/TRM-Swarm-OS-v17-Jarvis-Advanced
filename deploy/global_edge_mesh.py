"""
Swarm OS v12 - Global Edge Mesh
Latency-aware, multi-region traffic distribution layer.
"""

import logging
import random
from typing import Dict, List, Optional
from pydantic import BaseModel

logger = logging.getLogger("EdgeMesh")

class EdgeNode(BaseModel):
    node_id: str
    region: str
    latitude: float
    longitude: float
    health: str = "healthy"
    load: float = 0.0

class GlobalEdgeMesh:
    def __init__(self):
        self.nodes = {
            "us-east-1": EdgeNode(node_id="edge-na-1", region="NA", latitude=38.8, longitude=-77.0),
            "eu-west-1": EdgeNode(node_id="edge-eu-1", region="EU", latitude=53.3, longitude=-6.2),
            "ap-southeast-1": EdgeNode(node_id="edge-ap-1", region="APAC", latitude=1.3, longitude=103.8)
        }

    def get_optimal_node(self, client_region: str) -> Optional[EdgeNode]:
        """
        Routes traffic to the healthiest node in the closest region.
        """
        # Simple region-based routing for this implementation
        if client_region in self.nodes:
            node = self.nodes[client_region]
            if node.health == "healthy":
                return node
        
        # Fallback: pick any healthy node
        healthy_nodes = [n for n in self.nodes.values() if n.health == "healthy"]
        if healthy_nodes:
            return random.choice(healthy_nodes)
        
        return None

    def update_node_status(self, node_id: str, health: str, load: float):
        for node in self.nodes.values():
            if node.node_id == node_id:
                node.health = health
                node.load = load
                logger.info(f"[EdgeMesh] Node {node_id} status updated: {health} (load: {load})")
                return True
        return False

# Singleton
_edge_mesh = GlobalEdgeMesh()

def get_edge_mesh() -> GlobalEdgeMesh:
    return _edge_mesh
