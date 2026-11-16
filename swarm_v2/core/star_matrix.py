"""
Star Matrix Narrative Lattice — TRM Swarm OS (Project Continuum V5.3)

Implements a celestial node graph where each node represents a narrative
anchor in the swarm's collective consciousness. Connections between nodes
are weighted by a Gravitational Resonance Equation that blends semantic
similarity with positional mass attraction.

Singleton access via get_star_matrix().
"""

import logging
import math
import time
import hashlib
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from collections import deque

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# StarMatrixNode
# ---------------------------------------------------------------------------

class StarMatrixNode:
    """A celestial node in the narrative lattice."""

    __slots__ = (
        "node_id",
        "label",
        "position",
        "mass",
        "embedding",
        "linked_agents",
        "resonance_score",
        "last_sync",
    )

    def __init__(
        self,
        node_id: str,
        label: str,
        position: Tuple[float, float],
        mass: float,
        linked_agents: Optional[List[str]] = None,
        embedding: Optional[List[float]] = None,
        resonance_score: float = 0.0,
        last_sync: Optional[str] = None,
    ) -> None:
        self.node_id: str = node_id
        self.label: str = label
        self.position: Tuple[float, float] = position
        self.mass: float = mass
        self.embedding: List[float] = embedding if embedding is not None else []
        self.linked_agents: List[str] = linked_agents if linked_agents is not None else []
        self.resonance_score: float = resonance_score
        self.last_sync: str = last_sync or datetime.utcnow().isoformat()

    # -- serialisation -------------------------------------------------------

    def to_dict(self) -> Dict[str, Any]:
        """Return a JSON-serialisable dictionary representation."""
        return {
            "node_id": self.node_id,
            "label": self.label,
            "position": list(self.position),
            "mass": self.mass,
            "embedding": list(self.embedding),
            "linked_agents": list(self.linked_agents),
            "resonance_score": self.resonance_score,
            "last_sync": self.last_sync,
        }

    def __repr__(self) -> str:  # pragma: no cover
        return (
            f"StarMatrixNode(id={self.node_id!r}, label={self.label!r}, "
            f"mass={self.mass}, resonance={self.resonance_score:.3f})"
        )


# ---------------------------------------------------------------------------
# StarMatrix
# ---------------------------------------------------------------------------

class StarMatrix:
    """
    Narrative lattice of celestial nodes.

    The matrix maintains four canonical nodes and computes pairwise
    *Gravitational Resonance* scores that blend semantic cosine similarity
    with a mass-distance attraction term.

    Access the singleton instance via :func:`get_star_matrix`.
    """

    # Gravitational Resonance weights
    ALPHA: float = 0.6  # semantic similarity weight
    BETA: float = 0.4   # gravitational attraction weight

    def __init__(self) -> None:
        logger.info("Initialising StarMatrix narrative lattice …")

        # -- canonical nodes -------------------------------------------------
        self.nodes: Dict[str, StarMatrixNode] = {}
        self._init_default_nodes()

        # -- connections & external bindings ---------------------------------
        self.connections: List[dict] = []
        self._resonance_engine: Optional[Any] = None
        self._agent_mesh: Optional[Any] = None
        self._sync_history: deque = deque(maxlen=100)

        # Compute initial connection graph
        self.compute_all_connections()

        logger.info(
            "StarMatrix ready — %d nodes, %d connections.",
            len(self.nodes),
            len(self.connections),
        )

    # -- initialisation helpers ----------------------------------------------

    def _init_default_nodes(self) -> None:
        """Populate the lattice with the four canonical celestial nodes."""
        defaults: List[Dict[str, Any]] = [
            {
                "node_id": "alpha",
                "label": "The Alpha Singularity",
                "position": (50.0, 18.0),
                "mass": 10.0,
                "linked_agents": ["Architect", "Lead Developer"],
            },
            {
                "node_id": "monad",
                "label": "The Monad Anchor",
                "position": (20.0, 65.0),
                "mass": 8.0,
                "linked_agents": ["Research Analyst", "Data Engineer"],
            },
            {
                "node_id": "nexus",
                "label": "The Nexus Confluence",
                "position": (80.0, 65.0),
                "mass": 9.0,
                "linked_agents": ["QA Engineer", "Security Auditor"],
            },
            {
                "node_id": "gamma",
                "label": "The Unified Consciousness Core",
                "position": (50.0, 52.0),
                "mass": 14.0,
                "linked_agents": ["Orchestrator", "DevOps Engineer"],
            },
        ]
        for cfg in defaults:
            node = StarMatrixNode(**cfg)
            self.nodes[node.node_id] = node
            logger.debug("Registered node %r (%s).", node.node_id, node.label)

    # -- binding -------------------------------------------------------------

    def bind(
        self,
        resonance_engine: Optional[Any] = None,
        agent_mesh: Optional[Any] = None,
    ) -> None:
        """Bind external subsystem references for cross-module sync."""
        if resonance_engine is not None:
            self._resonance_engine = resonance_engine
            logger.info("StarMatrix bound to ResonanceEngine.")
        if agent_mesh is not None:
            self._agent_mesh = agent_mesh
            logger.info("StarMatrix bound to AgentMesh.")

    # -- resonance computation -----------------------------------------------

    @staticmethod
    def _cosine_similarity(a: List[float], b: List[float]) -> float:
        """Compute cosine similarity between two vectors."""
        if not a or not b or len(a) != len(b):
            return 0.5  # default when embeddings unavailable / mismatched
        dot = sum(x * y for x, y in zip(a, b))
        mag_a = math.sqrt(sum(x * x for x in a))
        mag_b = math.sqrt(sum(x * x for x in b))
        if mag_a == 0.0 or mag_b == 0.0:
            return 0.5
        return dot / (mag_a * mag_b)

    @staticmethod
    def _euclidean_distance(
        p1: Tuple[float, float],
        p2: Tuple[float, float],
    ) -> float:
        """Euclidean distance between two 2-D points, clamped to ≥ 1.0."""
        dx = p1[0] - p2[0]
        dy = p1[1] - p2[1]
        return max(math.sqrt(dx * dx + dy * dy), 1.0)

    def compute_gravitational_resonance(
        self,
        node_a_id: str,
        node_b_id: str,
    ) -> float:
        """
        Gravitational Resonance Equation:

            R(S₁, S₂) = α · Sim(E₁, E₂) + β · (M₁ · M₂) / d²

        Returns a float resonance score.  Raises ``KeyError`` if either
        node ID is unknown.
        """
        try:
            node_a = self.nodes[node_a_id]
            node_b = self.nodes[node_b_id]
        except KeyError as exc:
            logger.error("Unknown node id in resonance computation: %s", exc)
            raise

        sim = self._cosine_similarity(node_a.embedding, node_b.embedding)
        dist = self._euclidean_distance(node_a.position, node_b.position)
        gravitational = (node_a.mass * node_b.mass) / (dist * dist)
        resonance = self.ALPHA * sim + self.BETA * gravitational
        logger.debug(
            "Resonance(%s ↔ %s) = %.4f  [sim=%.3f, grav=%.4f, d=%.2f]",
            node_a_id,
            node_b_id,
            resonance,
            sim,
            gravitational,
            dist,
        )
        return resonance

    def compute_all_connections(self) -> List[dict]:
        """Compute resonance for every unique pair and cache the result."""
        connections: List[dict] = []
        node_ids = sorted(self.nodes.keys())
        for i, id_a in enumerate(node_ids):
            for id_b in node_ids[i + 1:]:
                try:
                    resonance = self.compute_gravitational_resonance(id_a, id_b)
                    connections.append(
                        {
                            "source": id_a,
                            "target": id_b,
                            "resonance": round(resonance, 6),
                            "timestamp": datetime.utcnow().isoformat(),
                        }
                    )
                except Exception:
                    logger.exception(
                        "Failed to compute resonance for pair (%s, %s).",
                        id_a,
                        id_b,
                    )
        self.connections = connections
        logger.info("Computed %d pairwise connections.", len(connections))
        return connections

    # -- resonance engine sync -----------------------------------------------

    def sync_with_resonance_cache(self) -> None:
        """Pull active dreams from a bound ResonanceEngine and refresh scores."""
        if self._resonance_engine is None:
            logger.warning(
                "sync_with_resonance_cache called but no ResonanceEngine is bound."
            )
            return

        try:
            active_dreams = getattr(
                self._resonance_engine, "active_dreams", None
            )
            if active_dreams is None:
                active_dreams = getattr(
                    self._resonance_engine, "get_active_dreams", lambda: []
                )()

            if not isinstance(active_dreams, (list, tuple)):
                logger.warning("active_dreams is not iterable — skipping sync.")
                return

            updated_nodes: List[str] = []
            for dream in active_dreams:
                focus_nodes: List[str] = []
                if isinstance(dream, dict):
                    focus_nodes = dream.get("focus_nodes", [])
                elif hasattr(dream, "focus_nodes"):
                    focus_nodes = dream.focus_nodes or []

                # Find the most relevant node whose linked_agents overlap
                for node in self.nodes.values():
                    overlap = set(node.linked_agents) & set(focus_nodes)
                    if overlap:
                        # Boost resonance_score (clamped to 1.0)
                        boost = min(len(overlap) * 0.1, 0.5)
                        node.resonance_score = min(
                            node.resonance_score + boost, 1.0
                        )
                        node.last_sync = datetime.utcnow().isoformat()
                        updated_nodes.append(node.node_id)

            # Recompute full connection graph after score updates
            self.compute_all_connections()

            # Record sync event
            sync_event = {
                "timestamp": datetime.utcnow().isoformat(),
                "dreams_processed": len(active_dreams),
                "nodes_updated": updated_nodes,
                "checksum": hashlib.sha256(
                    str(time.time_ns()).encode()
                ).hexdigest()[:12],
            }
            self._sync_history.append(sync_event)
            logger.info(
                "Resonance cache sync complete — %d dreams, %d node(s) updated.",
                len(active_dreams),
                len(updated_nodes),
            )

        except Exception:
            logger.exception("Error during resonance cache sync.")

    # -- query / export ------------------------------------------------------

    def get_lattice(self) -> dict:
        """Return the full lattice state as a serialisable dict."""
        return {
            "nodes": [node.to_dict() for node in self.nodes.values()],
            "connections": list(self.connections),
            "sync_count": len(self._sync_history),
        }

    def get_d3_overlay(self) -> dict:
        """
        Return a D3.js-consumable representation of the lattice.

        * ``nodes`` — list of {id, label, x, y, mass, resonance_score, linked_agents}
        * ``links`` — list of {source, target, strength}
        """
        d3_nodes: List[dict] = []
        for node in self.nodes.values():
            d3_nodes.append(
                {
                    "id": node.node_id,
                    "label": node.label,
                    "x": node.position[0],
                    "y": node.position[1],
                    "mass": node.mass,
                    "resonance_score": node.resonance_score,
                    "linked_agents": list(node.linked_agents),
                }
            )

        d3_links: List[dict] = []
        for conn in self.connections:
            d3_links.append(
                {
                    "source": conn["source"],
                    "target": conn["target"],
                    "strength": conn["resonance"],
                }
            )

        return {"nodes": d3_nodes, "links": d3_links}


# ---------------------------------------------------------------------------
# Singleton accessor
# ---------------------------------------------------------------------------

_star_matrix: Optional[StarMatrix] = None


def get_star_matrix() -> StarMatrix:
    """Return the singleton :class:`StarMatrix` instance (lazy-initialised)."""
    global _star_matrix
    if _star_matrix is None:
        _star_matrix = StarMatrix()
    return _star_matrix
