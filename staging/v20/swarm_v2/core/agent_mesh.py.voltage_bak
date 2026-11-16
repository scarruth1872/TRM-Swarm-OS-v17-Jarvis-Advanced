"""
P2P Agent Mesh — Phase 3: Distributed Consciousness
Each agent becomes its own micro-node in a peer-to-peer fabric.

Architecture:
- Every agent registers itself on the mesh with its capabilities
- Agents can discover peers and route tasks directly (no central orchestrator needed)
- If the main gateway (port 8000) goes down, agents keep running
- Gossip-based heartbeat keeps the mesh topology alive

Security: All deserialization is validated through Pydantic v2 schema enforcement.
"""

import asyncio
import json
import os
import time
import hashlib
import logging
from typing import Dict, List, Optional, Any
from datetime import datetime

from pydantic import BaseModel, Field, field_validator, model_validator

from swarm_v2.skills.embedding_skill import FastEmbeddingSkill

# Configure module logger
logger = logging.getLogger(__name__)

MESH_DIR = "swarm_v2_mesh"
os.makedirs(MESH_DIR, exist_ok=True)


# ──────────────────────────────────────────────
# Pydantic v2 Validation Schema
# ──────────────────────────────────────────────

class MeshNodeSchema(BaseModel):
    """Pydantic v2 validated schema for MeshNode deserialization.
    
    Enforces strict typing, value constraints, and cross-field validation
    to prevent type confusion, attribute injection, and data corruption attacks.
    """
    
    node_id: str = Field(
        ...,
        min_length=1,
        max_length=128,
        pattern=r'^[a-zA-Z0-9_-]+$',
        description="Unique node identifier (alphanumeric, hyphens, underscores only)"
    )
    name: str = Field(
        ...,
        min_length=1,
        max_length=256,
        description="Human-readable node name"
    )
    role: str = Field(
        ...,
        min_length=1,
        max_length=64,
        description="Node role (e.g., worker, coordinator, gateway)"
    )
    host: str = Field(
        default="127.0.0.1",
        pattern=r'^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$|^localhost$',
        description="IPv4 address or 'localhost'"
    )
    port: int = Field(
        default=0,
        ge=0,
        le=65535,
        description="Network port number"
    )
    specialties: List[str] = Field(
        default_factory=list,
        max_length=50,
        description="List of node specialties"
    )
    skills: List[str] = Field(
        default_factory=list,
        max_length=50,
        description="List of node skills"
    )
    hardware: Dict[str, Any] = Field(
        default_factory=dict,
        description="Hardware capabilities dictionary"
    )
    status: str = Field(
        default="online",
        pattern=r'^(online|offline|busy|error)$',
        description="Current node status"
    )
    last_heartbeat: float = Field(
        default_factory=time.time,
        description="Unix timestamp of last heartbeat"
    )
    joined_at: str = Field(
        default_factory=lambda: datetime.now().isoformat(),
        description="ISO 8601 timestamp of node join time"
    )
    task_count: int = Field(
        default=0,
        ge=0,
        description="Number of tasks processed"
    )
    peer_connections: int = Field(
        default=0,
        ge=0,
        description="Number of active peer connections"
    )

    @field_validator('last_heartbeat')
    @classmethod
    def validate_heartbeat_timestamp(cls, v: float) -> float:
        """Ensure heartbeat is a valid Unix timestamp (not future by more than 5 minutes)."""
        now = time.time()
        if v > now + 300:  # Allow 5-minute clock skew
            raise ValueError(f"Heartbeat timestamp {v} is too far in the future (max {now + 300})")
        if v < 946684800:  # Year 2000 minimum
            raise ValueError(f"Heartbeat timestamp {v} is too far in the past (min 946684800)")
        return v

    @field_validator('host')
    @classmethod
    def validate_host_format(cls, v: str) -> str:
        """Additional host validation beyond regex."""
        if v == "localhost":
            return v
        parts = v.split('.')
        if len(parts) != 4:
            raise ValueError(f"Invalid IP address format: {v}")
        for part in parts:
            num = int(part)
            if num < 0 or num > 255:
                raise ValueError(f"IP octet out of range: {part}")
        return v

    @model_validator(mode='after')
    def validate_node_consistency(self) -> 'MeshNodeSchema':
        """Cross-field validation: ensure node_id is consistent with name."""
        if self.node_id and self.name:
            # Sanity check: node_id should be a sanitized version of name
            expected_id = self.name.lower().replace(' ', '_')[:64]
            # This is informational; we don't enforce it strictly for backward compatibility
            if self.node_id != expected_id and not self.node_id.startswith(expected_id[:32]):
                logger.warning(
                    f"Node ID '{self.node_id}' does not match expected pattern from name '{self.name}'"
                )
        return self


# ──────────────────────────────────────────────
# MeshNode Class
# ──────────────────────────────────────────────

class MeshNode:
    """Represents a single agent node in the P2P mesh.
    
    All construction and deserialization is validated through Pydantic v2 schema.
    Backward-compatible with existing code that uses from_dict() and to_dict().
    """

    __slots__ = (
        'node_id', 'name', 'role', 'host', 'port',
        'specialties', 'skills', 'hardware', 'status',
        'last_heartbeat', 'joined_at', 'task_count', 'peer_connections'
    )

    def __init__(self, node_id: str, name: str, role: str,
                 host: str = "127.0.0.1", port: int = 0,
                 specialties: List[str] = None,
                 skills: List[str] = None,
                 hardware: dict = None):
        # Validate constructor arguments through schema
        validated = MeshNodeSchema(
            node_id=node_id,
            name=name,
            role=role,
            host=host,
            port=port,
            specialties=specialties or [],
            skills=skills or [],
            hardware=hardware or {}
        )
        self.node_id = validated.node_id
        self.name = validated.name
        self.role = validated.role
        self.host = validated.host
        self.port = validated.port
        self.specialties = validated.specialties
        self.skills = validated.skills
        self.hardware = validated.hardware
        self.status = validated.status
        self.last_heartbeat = validated.last_heartbeat
        self.joined_at = validated.joined_at
        self.task_count = validated.task_count
        self.peer_connections = validated.peer_connections

    def to_dict(self) -> dict:
        """Serialize to dict (backward-compatible format)."""
        return {
            "node_id": self.node_id,
            "name": self.name,
            "role": self.role,
            "host": self.host,
            "port": self.port,
            "specialties": self.specialties,
            "skills": self.skills,
            "hardware": self.hardware,
            "status": self.status,
            "last_heartbeat": self.last_heartbeat,
            "joined_at": self.joined_at,
            "task_count": self.task_count,
            "peer_connections": self.peer_connections,
            "latency_ms": round((time.time() - self.last_heartbeat) * 1000, 1),
        }

    @classmethod
    def from_dict(cls, data: dict) -> "MeshNode":
        """Factory method with strict validation via Pydantic v2 schema.
        
        Args:
            data: Dictionary containing node data.
            
        Returns:
            MeshNode instance with validated fields.
            
        Raises:
            ValueError: If data fails schema validation.
        """
        try:
            validated = MeshNodeSchema(**data)
            # Use __new__ to bypass __init__ validation (already done via schema)
            node = cls.__new__(cls)
            node.node_id = validated.node_id
            node.name = validated.name
            node.role = validated.role
            node.host = validated.host
            node.port = validated.port
            node.specialties = validated.specialties
            node.skills = validated.skills
            node.hardware = validated.hardware
            node.status = validated.status
            node.last_heartbeat = validated.last_heartbeat
            node.joined_at = validated.joined_at
            node.task_count = validated.task_count
            node.peer_connections = validated.peer_connections
            return node
        except Exception as e:
            logger.error(f"Failed to deserialize MeshNode: {e}")
            raise ValueError(f"Invalid mesh node data: {e}") from e

    def heartbeat(self):
        """Update heartbeat timestamp and set status to online."""
        self.last_heartbeat = time.time()
        self.status = "online"

    @property
    def is_alive(self) -> bool:
        """Check if node is alive based on heartbeat timeout (10 minutes)."""
        return (time.time() - self.last_heartbeat) < 600

    def __repr__(self) -> str:
        return f"MeshNode(id={self.node_id}, role={self.role}, status={self.status})"


# ──────────────────────────────────────────────
# AgentMesh Class
# ──────────────────────────────────────────────

class AgentMesh:
    """
    The P2P mesh fabric. Manages node discovery, routing, and health.
    Survives orchestrator failures via decentralized state.
    
    Features:
    - Validated node registration and deserialization
    - Gossip-based heartbeat propagation
    - Topology version tracking for conflict resolution
    - Graceful handling of corrupted state files
    """

    def __init__(self):
        self.nodes: Dict[str, MeshNode] = {}
        self.message_log: List[dict] = []
        self.topology_version = 0
        # Load topology version but don't restore stale nodes
        # Nodes will be re-registered on each startup
        self._load_topology_version()

    def _load_topology_version(self):
        """Load only the topology version from persisted state."""
        state_path = os.path.join(MESH_DIR, "mesh_state.json")
        if os.path.exists(state_path):
            try:
                with open(state_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                self.topology_version = data.get("topology_version", 0)
                logger.info(f"Loaded topology version {self.topology_version}")
            except (json.JSONDecodeError, IOError) as e:
                logger.warning(f"Failed to load topology version: {e}")

    def _load_state(self):
        """Load persisted mesh state with validation."""
        state_path = os.path.join(MESH_DIR, "mesh_state.json")
        if os.path.exists(state_path):
            try:
                with open(state_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                
                # Validate and restore nodes
                nodes_data = data.get("nodes", [])
                restored_count = 0
                for node_data in nodes_data:
                    try:
                        node = MeshNode.from_dict(node_data)
                        if node.is_alive:
                            self.nodes[node.node_id] = node
                            restored_count += 1
                        else:
                            logger.info(f"Skipping stale node {node.node_id}")
                    except ValueError as e:
                        logger.warning(f"Skipping invalid node data: {e}")
                
                self.topology_version = data.get("topology_version", 0)
                logger.info(f"Restored {restored_count} nodes from state (version {self.topology_version})")
                
            except (json.JSONDecodeError, IOError) as e:
                logger.error(f"Failed to load mesh state: {e}")
                # Don't raise - mesh can start fresh

    def _save_state(self):
        """Persist current mesh state to disk."""
        state_path = os.path.join(MESH_DIR, "mesh_state.json")
        try:
            state = {
                "nodes": [node.to_dict() for node in self.nodes.values()],
                "topology_version": self.topology_version,
                "saved_at": datetime.now().isoformat()
            }
            with open(state_path, "w", encoding="utf-8") as f:
                json.dump(state, f, indent=2)
            logger.debug(f"Saved mesh state with {len(self.nodes)} nodes")
        except IOError as e:
            logger.error(f"Failed to save mesh state: {e}")

    def register_node(self, node: MeshNode) -> bool:
        """Register a node in the mesh.
        
        Args:
            node: Validated MeshNode instance.
            
        Returns:
            True if node was registered, False if already exists.
        """
        if node.node_id in self.nodes:
            logger.debug(f"Node {node.node_id} already registered, updating heartbeat")
            self.nodes[node.node_id].heartbeat()
            return False
        
        self.nodes[node.node_id] = node
        self.topology_version += 1
        logger.info(f"Registered node {node.node_id} ({node.role})")
        return True

    def unregister_node(self, node_id: str) -> bool:
        """Remove a node from the mesh.
        
        Args:
            node_id: ID of node to remove.
            
        Returns:
            True if node was removed, False if not found.
        """
        if node_id in self.nodes:
            del self.nodes[node_id]
            self.topology_version += 1
            logger.info(f"Unregistered node {node_id}")
            return True
        return False

    def get_node(self, node_id: str) -> Optional[MeshNode]:
        """Get a node by ID.
        
        Args:
            node_id: ID of node to retrieve.
            
        Returns:
            MeshNode if found, None otherwise.
        """
        return self.nodes.get(node_id)

    def get_alive_nodes(self) -> List[MeshNode]:
        """Get all currently alive nodes.
        
        Returns:
            List of MeshNode instances that are alive.
        """
        return [node for node in self.nodes.values() if node.is_alive]

    def get_nodes_by_role(self, role: str) -> List[MeshNode]:
        """Get all nodes with a specific role.
        
        Args:
            role: Role to filter by.
            
        Returns:
            List of MeshNode instances matching the role.
        """
        return [node for node in self.nodes.values() if node.role == role]

    def get_nodes_by_specialty(self, specialty: str) -> List[MeshNode]:
        """Get all nodes with a specific specialty.
        
        Args:
            specialty: Specialty to filter by.
            
        Returns:
            List of MeshNode instances matching the specialty.
        """
        return [
            node for node in self.nodes.values()
            if specialty in node.specialties
        ]

    def propagate_heartbeat(self, node_id: str) -> bool:
        """Propagate heartbeat for a node.
        
        Args:
            node_id: ID of node to heartbeat.
            
        Returns:
            True if heartbeat was propagated, False if node not found.
        """
        node = self.nodes.get(node_id)
        if node:
            node.heartbeat()
            return True
        return False

    def cleanup_stale_nodes(self, timeout: int = 600) -> int:
        """Remove nodes that haven't heartbeated within timeout.
        
        Args:
            timeout: Timeout in seconds (default 600).
            
        Returns:
            Number of nodes removed.
        """
        stale_nodes = [
            node_id for node_id, node in self.nodes.items()
            if (time.time() - node.last_heartbeat) > timeout
        ]
        for node_id in stale_nodes:
            del self.nodes[node_id]
            logger.info(f"Removed stale node {node_id}")
        
        if stale_nodes:
            self.topology_version += 1
        
        return len(stale_nodes)

    def get_topology_snapshot(self) -> dict:
        """Get a snapshot of the current mesh topology.
        
        Returns:
            Dictionary with topology data.
        """
        return {
            "version": self.topology_version,
            "node_count": len(self.nodes),
            "alive_count": len(self.get_alive_nodes()),
            "nodes": [node.to_dict() for node in self.nodes.values()],
            "connections": self._compute_connections(),
            "timestamp": datetime.now().isoformat()
        }

    def get_topology(self) -> dict:
        """Alias for backward compatibility with monitoring/insights tools."""
        return self.get_topology_snapshot()

    def find_best_node(self, task: str, required_specialty: str = None) -> Optional[MeshNode]:
        """Find the best alive node for a given task using skill matching."""
        candidates = []
        task_lower = task.lower()

        for node in self.nodes.values():
            if not node.is_alive:
                continue

            score = 0
            # Specialty matching
            for spec in node.specialties:
                if spec.lower() in task_lower:
                    score += 10
            # Skill matching
            for skill in node.skills:
                if skill.lower() in task_lower:
                    score += 5
            # Role matching
            if node.role.lower() in task_lower:
                score += 8
            # Required specialty filter
            if required_specialty:
                if required_specialty in node.specialties:
                    score += 20
                else:
                    continue
            # Prefer less busy nodes
            score -= node.task_count * 0.1

            if score > 0:
                candidates.append((score, node))

        if not candidates:
            # Return any alive node as fallback
            alive = [n for n in self.nodes.values() if n.is_alive]
            return alive[0] if alive else None

        candidates.sort(key=lambda x: x[0], reverse=True)
        
        # If the top score is low, try semantic matching as a tie-breaker or fallback
        if candidates[0][0] < 10:
            try:
                embedder = FastEmbeddingSkill()
                task_emb = embedder.embed_text(task_lower)
                if task_emb:
                    semantic_candidates = []
                    for _, node in candidates:
                        # Combine specialties and skills into a node description
                        node_desc = f"{node.role} {node.name} {' '.join(node.specialties)} {' '.join(node.skills)}"
                        node_emb = embedder.embed_text(node_desc.lower())
                        score = embedder.cosine_similarity(task_emb, node_emb)
                        semantic_candidates.append((score, node))
                    
                    semantic_candidates.sort(key=lambda x: x[0], reverse=True)
                    return semantic_candidates[0][1]
            except Exception as e:
                logger.error(f"Semantic routing failed: {e}")

        return candidates[0][1]

    async def route_task(self, task: str, target_node_id: str = None,
                          required_specialty: str = None) -> dict:
        """Route a task to a node on the mesh (P2P style)."""
        if target_node_id and target_node_id in self.nodes:
            node = self.nodes[target_node_id]
        else:
            node = self.find_best_node(task, required_specialty)

        if not node:
            return {"error": "No suitable node found on the mesh", "task": task}

        node.task_count += 1
        self._save_state()

        self.message_log.append({
            "type": "task_routed",
            "task": task[:80],
            "target_node": node.name,
            "target_role": node.role,
            "node_id": node.node_id,
            "timestamp": datetime.now().isoformat(),
        })

        return {
            "routed_to": node.to_dict(),
            "task": task,
            "mesh_version": self.topology_version,
        }

    def _compute_connections(self) -> List[dict]:
        """Compute logical connections between nodes (for visualization)."""
        connections = []
        node_list = list(self.nodes.values())
        for i, a in enumerate(node_list):
            for b in node_list[i + 1:]:
                strength = 0
                shared = set(a.specialties) & set(b.specialties)
                
                # Heuristic 1: Shared Specialties (High Strength)
                if shared:
                    strength += len(shared) * 2
                
                # Heuristic 2: Role Proximity (Dynamic collaboration)
                dev_words = {"developer", "architect", "engineer", "logic"}
                if (any(w in a.role.lower() for w in dev_words) and 
                    any(w in b.role.lower() for w in dev_words)):
                    strength += 1
                
                # Heuristic 3: Architect Star Topology (Central hub)
                if a.role == "Architect" or b.role == "Architect":
                    strength += 1

                if strength > 0:
                    connections.append({
                        "from": a.node_id,
                        "to": b.node_id,
                        "shared_specialties": list(shared),
                        "strength": strength,
                    })
        return connections

    def get_stats(self) -> dict:
        """Get stats about the agent mesh."""
        alive = [n for n in self.nodes.values() if n.is_alive]
        return {
            "total_nodes": len(self.nodes),
            "alive_nodes": len(alive),
            "topology_version": self.topology_version,
            "total_tasks_routed": sum(n.task_count for n in self.nodes.values()),
            "total_messages": len(self.message_log),
            "connections": len(self._compute_connections()),
        }

    def get_message_log(self, limit: int = 20) -> List[dict]:
        """Get the message log of the mesh."""
        return self.message_log[-limit:]

    def reconfigure_mesh(self) -> dict:
        """
        Phase 7: Real-time Mesh Reconfiguration.
        Detects failed nodes, redistributes their specialties to surviving nodes,
        and logs reconfiguration events.
        """
        alive = [n for n in self.nodes.values() if n.is_alive]
        failed = [n for n in self.nodes.values() if not n.is_alive]

        if not failed:
            return {"action": "none", "alive": len(alive), "failed": 0}

        redistributed = 0
        for dead_node in failed:
            orphan_specialties = dead_node.specialties
            if orphan_specialties and alive:
                # Distribute orphan specialties round-robin to alive nodes
                for i, spec in enumerate(orphan_specialties):
                    target = alive[i % len(alive)]
                    if spec not in target.specialties:
                        target.specialties.append(spec)
                        redistributed += 1

            # Log the reconfiguration event
            self.message_log.append({
                "type": "mesh_reconfiguration",
                "failed_node": dead_node.name,
                "failed_node_id": dead_node.node_id,
                "orphan_specialties": orphan_specialties,
                "redistributed_to": [n.name for n in alive],
                "timestamp": datetime.now().isoformat(),
            })

            # Mark as explicitly offline (keep in registry for recovery)
            dead_node.status = "offline"

        self.topology_version += 1
        self._save_state()

        return {
            "action": "reconfigured",
            "alive": len(alive),
            "failed": len(failed),
            "specialties_redistributed": redistributed,
            "topology_version": self.topology_version
        }

    def __len__(self) -> int:
        return len(self.nodes)

    def __contains__(self, node_id: str) -> bool:
        return node_id in self.nodes

_agent_mesh = None

def get_agent_mesh() -> AgentMesh:
    global _agent_mesh
    if _agent_mesh is None:
        _agent_mesh = AgentMesh()
    return _agent_mesh

