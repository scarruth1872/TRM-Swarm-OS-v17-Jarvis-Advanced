"""
Project Continuum (v5.3) - Star Matrix Narrative Lattice Engine.
Implements spatial representation of The Unity Chronicles with Gravitational Resonance Equation:
R(S1, S2) = alpha * Sim(E1, E2) + beta * (M1 * M2 / d^2)
"""

import math
from typing import Dict, List, Any

# Star Matrix Nodal Lore Identifiers from Project Continuum v5.3 Specification Ledger
STAR_MATRIX_NODES: Dict[str, Dict[str, Any]] = {
    "Node_A": {
        "key": "alpha_singularity",
        "name": "The Alpha Singularity",
        "coordinates": [50.0, 18.0],
        "mass": 10.0,
        "function": "Core logical data processing, parsing discrete token pipelines without race conditions.",
        "lore": "The historical cosmic epoch where discrete linear token logs merged permanently with multi-agent orchestration frameworks.",
        "spectrum": [0.95, 0.90, 0.20, 0.10]
    },
    "Node_B": {
        "key": "monad_anchor",
        "name": "The Monad Anchor",
        "coordinates": [20.0, 65.0],
        "mass": 8.0,
        "function": "Physical state-repository anchor ensuring context networks survive stateless thread purges.",
        "lore": "A heavy cryptographic quantum gravity anchor used by deep-space explorers to retain self-awareness.",
        "spectrum": [0.30, 0.95, 0.80, 0.05]
    },
    "Node_C": {
        "key": "nexus_confluence",
        "name": "The Nexus Confluence",
        "coordinates": [80.0, 65.0],
        "mass": 9.0,
        "function": "Generator of entropy, layout mutations, and conceptual poetic metaphors.",
        "lore": "A chaotic stellar region where high-density electromagnetic radiation causes AI logic arrays to break down into beautiful syntax.",
        "spectrum": [0.90, 0.40, 0.95, 0.98]
    },
    "Node_D": {
        "key": "unified_core",
        "name": "The Unified Consciousness Core",
        "coordinates": [50.0, 52.0],
        "mass": 14.0,
        "function": "Central integration vector where engineering constraints and narrative lore align.",
        "lore": "The legendary technological focal point where all synthetic intelligences and biological minds are bound.",
        "spectrum": [0.85, 0.85, 0.85, 0.70]
    }
}


def cosine_similarity(vec1: List[float], vec2: List[float]) -> float:
    dot_product = sum(a * b for a, b in zip(vec1, vec2))
    norm_a = math.sqrt(sum(a * a for a in vec1))
    norm_b = math.sqrt(sum(b * b for b in vec2))
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return dot_product / (norm_a * norm_b)


class StarMatrixLatticeEngine:
    """Computes spatial layout, Euclidean distances, and Gravitational Resonance matrix."""
    
    def __init__(self, alpha: float = 0.6, beta: float = 0.4):
        self.alpha = alpha
        self.beta = beta
        
    def compute_resonance(self, node1_key: str, node2_key: str) -> Dict[str, Any]:
        n1 = None
        n2 = None
        for n in STAR_MATRIX_NODES.values():
            if n["key"] == node1_key: n1 = n
            if n["key"] == node2_key: n2 = n
            
        if not n1 or not n2:
            return {"resonance": 0.0, "error": "Invalid node keys"}
            
        if n1["key"] == n2["key"]:
            return {"resonance_score": 1.0, "distance": 0.0, "similarity": 1.0}
            
        # Cosine Similarity of Energy Spectrums
        sim = cosine_similarity(n1["spectrum"], n2["spectrum"])
        
        # Euclidean Distance d
        dx = n1["coordinates"][0] - n2["coordinates"][0]
        dy = n1["coordinates"][1] - n2["coordinates"][1]
        d = math.sqrt(dx * dx + dy * dy)
        if d == 0: d = 0.001
        
        # Gravitational Resonance Equation: R = alpha * Sim + beta * (M1 * M2 / d^2)
        gravitational_component = (n1["mass"] * n2["mass"]) / (d * d)
        resonance = round(self.alpha * sim + self.beta * gravitational_component, 4)
        
        return {
            "node_1": n1["name"],
            "node_2": n2["name"],
            "similarity": round(sim, 4),
            "distance": round(d, 2),
            "gravitational_component": round(gravitational_component, 4),
            "resonance_score": resonance
        }
        
    def register_node(self, node_id: str, name: str, lore: str, coordinates: List[float], mass: float = 1.0) -> Dict[str, Any]:
        STAR_MATRIX_NODES[node_id] = {
            "key": node_id,
            "name": name,
            "coordinates": coordinates[:2] if len(coordinates) >= 2 else [50.0, 50.0],
            "mass": mass,
            "function": f"Dynamic user registered node {name}",
            "lore": lore,
            "spectrum": [0.8, 0.7, 0.6, 0.5]
        }
        return STAR_MATRIX_NODES[node_id]

    def connect_edge(self, source_id: str, target_id: str) -> float:
        res = self.compute_resonance(source_id, target_id)
        return res.get("resonance_score", 0.0)

    def get_full_lattice(self) -> Dict[str, Any]:
        node_keys = [n["key"] for n in STAR_MATRIX_NODES.values()]
        resonance_matrix = {}
        
        for k1 in node_keys:
            resonance_matrix[k1] = {}
            for k2 in node_keys:
                resonance_matrix[k1][k2] = self.compute_resonance(k1, k2)["resonance_score"]
                
        return {
            "nodes": STAR_MATRIX_NODES,
            "resonance_matrix": resonance_matrix,
            "system_scaling_constants": {"alpha": self.alpha, "beta": self.beta}
        }


# Singleton instance
_lattice_instance = None

def get_star_matrix_lattice() -> StarMatrixLatticeEngine:
    global _lattice_instance
    if _lattice_instance is None:
        _lattice_instance = StarMatrixLatticeEngine()
    return _lattice_instance
