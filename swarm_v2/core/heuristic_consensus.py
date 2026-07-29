import logging
import time
from typing import Dict, List, Optional, Any
from datetime import datetime

logger = logging.getLogger(__name__)

class HeuristicConsensusLayer:
    """
    Heuristic Consensus Layer for TRM Swarm OS.
    Handles real-time task distribution based on multi-factor scores:
    specialty matching, health metrics, node availability, and skill proficiency.
    """
    WEIGHTS = {
        "skill_match": 0.35,
        "health_score": 0.25,
        "availability": 0.20,
        "proficiency": 0.20
    }
    MAX_CONCURRENT_TASKS = 10

    def __init__(self):
        self._agent_mesh = None
        self._self_healing = None
        self._skill_matrix = None
        self._routing_log: List[dict] = []

    def bind(self, agent_mesh, self_healing, skill_matrix):
        """Bind references to other orchestration singletons."""
        self._agent_mesh = agent_mesh
        self._self_healing = self_healing
        self._skill_matrix = skill_matrix

    def score_agent(self, node, task: str) -> float:
        """Compute the weighted multi-factor capability score for an agent node."""
        task_lower = task.lower()

        # 1. Skill Match Score (0.0 to 1.0)
        raw_match = 0
        for spec in node.specialties:
            if spec.lower() in task_lower:
                raw_match += 10
        for skill in node.skills:
            if skill.lower() in task_lower:
                raw_match += 5
        if node.role.lower() in task_lower:
            raw_match += 8
        skill_match = min(raw_match / 30.0, 1.0)

        # 2. Health Score (0.0 to 1.0)
        if self._self_healing:
            health = self._self_healing.compute_health_score(node.role)
        else:
            health = 1.0 if node.is_alive else 0.0

        # 3. Availability Score (0.0 to 1.0)
        availability = 1.0 - (node.task_count / max(self.MAX_CONCURRENT_TASKS, 1))
        availability = max(0.0, min(1.0, availability))

        # 4. Proficiency Score (0.0 to 1.0)
        proficiency = 0.5  # Default neutral
        if self._skill_matrix and node.role in self._skill_matrix.proficiency_matrix:
            proficiencies = self._skill_matrix.proficiency_matrix[node.role]
            # Match against task key words to see if we can find a matching skill proficiency
            matched_profs = [v for k, v in proficiencies.items() if k.lower() in task_lower]
            if matched_profs:
                # Normalizing prof value to [0.0, 1.0] assuming 3.0 is max score
                proficiency = min(max(matched_profs) / 3.0, 1.0)
            elif proficiencies:
                # Fallback to max general proficiency / 6.0 (reduced weight if not direct match)
                proficiency = min(max(proficiencies.values()) / 6.0, 0.5)

        # Compute weighted sum
        score = (
            skill_match * self.WEIGHTS["skill_match"] +
            health * self.WEIGHTS["health_score"] +
            availability * self.WEIGHTS["availability"] +
            proficiency * self.WEIGHTS["proficiency"]
        )
        return round(score, 4)

    async def route(self, task: str, target_node_id: str = None, required_specialty: str = None) -> dict:
        """Route task to candidate node via multi-factor heuristic voting consensus."""
        if not self._agent_mesh:
            return {"error": "Mesh not bound to HeuristicConsensusLayer", "task": task}

        # 1. Explicit Routing Check
        if target_node_id and target_node_id in self._agent_mesh.nodes:
            node = self._agent_mesh.nodes[target_node_id]
            node.task_count += 1
            self._agent_mesh._save_state()
            return {
                "routed_to": node.to_dict(),
                "task": task,
                "mesh_version": self._agent_mesh.topology_version,
                "routing_method": "direct"
            }

        # 2. Get Candidates
        alive_nodes = self._agent_mesh.get_alive_nodes()
        if not alive_nodes:
            return {"error": "No alive nodes found on the mesh", "task": task}

        if required_specialty:
            candidates = [n for n in alive_nodes if required_specialty in n.specialties]
        else:
            candidates = alive_nodes

        if not candidates:
            # Fallback to all alive nodes if specialty filter returned empty
            candidates = alive_nodes

        # 3. Score Candidates
        scored_candidates = []
        for node in candidates:
            score = self.score_agent(node, task)
            scored_candidates.append((score, node))

        scored_candidates.sort(key=lambda x: x[0], reverse=True)
        best_score, best_node = scored_candidates[0]

        best_node.task_count += 1
        self._agent_mesh._save_state()

        decision = {
            "task": task[:80],
            "routed_to": best_node.role,
            "node_id": best_node.node_id,
            "heuristic_score": best_score,
            "timestamp": datetime.now().isoformat()
        }
        self._routing_log.append(decision)
        if len(self._routing_log) > 50:
            self._routing_log.pop(0)

        return {
            "routed_to": best_node.to_dict(),
            "task": task,
            "mesh_version": self._agent_mesh.topology_version,
            "heuristic_scores": {node.role: round(score, 3) for score, node in scored_candidates[:5]},
            "routing_method": "heuristic_consensus"
        }

    def get_routing_log(self, limit: int = 20) -> List[dict]:
        return self._routing_log[-limit:]

    def get_stats(self) -> dict:
        return {
            "total_routed": len(self._routing_log),
            "weights": self.WEIGHTS,
            "recent_decisions": self._routing_log[-5:]
        }

_instance = None

def get_heuristic_consensus() -> HeuristicConsensusLayer:
    global _instance
    if _instance is None:
        _instance = HeuristicConsensusLayer()
    return _instance
