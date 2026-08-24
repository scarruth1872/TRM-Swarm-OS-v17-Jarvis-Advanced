import logging
import time
import math
import json
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from collections import defaultdict

logger = logging.getLogger(__name__)

class SkillMatrixSync:
    """
    Skill Matrix ↔ Memory Graph Bridge.
    Synchronizes LearnedSkill objects into global memory + CQS knowledge graph,
    and calculates agent-skill proficiency matrices.
    """
    def __init__(self):
        self.proficiency_matrix: Dict[str, Dict[str, float]] = defaultdict(dict)  # role -> {skill -> prof}
        self._usage_tracker: Dict[str, Dict[str, dict]] = defaultdict(lambda: defaultdict(lambda: {"uses": 0, "successes": 0}))
        self._skill_events: List[dict] = []
        self._global_memory = None
        self._cqs_service = None
        self._spatial_broadcaster = None

    def bind(self, global_memory, cqs_service, spatial_broadcaster):
        """Bind references to global memory, knowledge graph, and spatial broadcaster."""
        self._global_memory = global_memory
        self._cqs_service = cqs_service
        self._spatial_broadcaster = spatial_broadcaster

    def on_skill_created(self, skill):
        """Handle new skill creation event."""
        try:
            # 1. Index into global memory vector store (ChromaDB)
            if self._global_memory:
                self._global_memory.contribute(
                    content=f"Skill: {skill.skill_name}\nDescription: {skill.description}\nInstructions: {skill.instructions}",
                    author="SkillMatrixSync",
                    author_role="system",
                    memory_type="skill_definition",
                    tags=["skill_matrix", skill.skill_name, skill.source or "unknown"]
                )
            
            # 2. Inject triples into CQS Knowledge Graph
            if self._cqs_service:
                self._cqs_service.ingest_triples([
                    {
                        "subject": skill.skill_name,
                        "subject_type": "skill",
                        "predicate": "LEARNED_FROM",
                        "object": skill.source or "unknown",
                        "object_type": "source",
                        "timestamp": time.time()
                    }
                ])
                for path, method in skill.endpoints.items():
                    self._cqs_service.ingest_triples([
                        {
                            "subject": skill.skill_name,
                            "subject_type": "skill",
                            "predicate": "PROVIDES",
                            "object": f"{method} {path}",
                            "object_type": "endpoint",
                            "timestamp": time.time()
                        }
                    ])

            # 3. Notify the spatial broadcaster
            if self._spatial_broadcaster:
                self._spatial_broadcaster.push_event({
                    "event_type": "skill_matrix_updated",
                    "action": "created",
                    "skill_name": skill.skill_name,
                    "timestamp": time.time()
                })

            self._skill_events.append({
                "action": "created",
                "skill_name": skill.skill_name,
                "timestamp": datetime.now().isoformat()
            })
            logger.info(f"[SkillSync] Synced skill creation '{skill.skill_name}' across collective memory.")
        except Exception as e:
            logger.error(f"[SkillSync] Error during on_skill_created for {skill.skill_name}: {e}")

    def on_skill_evolved(self, skill):
        """Handle skill evolution/synthesis update."""
        try:
            if self._global_memory:
                self._global_memory.contribute(
                    content=f"Skill (Evolved): {skill.skill_name}\nDescription: {skill.description}\nInstructions: {skill.instructions}",
                    author="SkillMatrixSync",
                    author_role="system",
                    memory_type="skill_definition",
                    tags=["skill_matrix", skill.skill_name, "evolved", skill.source or "unknown"]
                )

            if self._spatial_broadcaster:
                self._spatial_broadcaster.push_event({
                    "event_type": "skill_matrix_updated",
                    "action": "evolved",
                    "skill_name": skill.skill_name,
                    "timestamp": time.time()
                })

            self._skill_events.append({
                "action": "evolved",
                "skill_name": skill.skill_name,
                "timestamp": datetime.now().isoformat()
            })
            logger.info(f"[SkillSync] Synced skill evolution '{skill.skill_name}' across collective memory.")
        except Exception as e:
            logger.error(f"[SkillSync] Error during on_skill_evolved for {skill.skill_name}: {e}")

    def on_skill_forgotten(self, skill_name: str):
        """Handle skill deletion event."""
        try:
            if self._spatial_broadcaster:
                self._spatial_broadcaster.push_event({
                    "event_type": "skill_matrix_updated",
                    "action": "forgotten",
                    "skill_name": skill_name,
                    "timestamp": time.time()
                })

            # Clean local proficiency references
            for role in list(self.proficiency_matrix.keys()):
                if skill_name in self.proficiency_matrix[role]:
                    del self.proficiency_matrix[role][skill_name]
                if skill_name in self._usage_tracker[role]:
                    del self._usage_tracker[role][skill_name]

            self._skill_events.append({
                "action": "forgotten",
                "skill_name": skill_name,
                "timestamp": datetime.now().isoformat()
            })
            logger.info(f"[SkillSync] Cleaned forgotten skill '{skill_name}' from live matrices.")
        except Exception as e:
            logger.error(f"[SkillSync] Error during on_skill_forgotten for {skill_name}: {e}")

    def record_skill_usage(self, agent_role: str, skill_name: str, success: bool):
        """Record skill execution outcome and update proficiency score."""
        tracker = self._usage_tracker[agent_role][skill_name]
        tracker["uses"] += 1
        if success:
            tracker["successes"] += 1

        success_rate = tracker["successes"] / max(tracker["uses"], 1)
        # Calculate proficiency heuristic: logarithmic scaling with usage count * success rate
        proficiency = success_rate * math.log2(tracker["uses"] + 1)
        self.proficiency_matrix[agent_role][skill_name] = round(proficiency, 3)

        # Inject usage trace into CQS Graph
        if self._cqs_service:
            try:
                self._cqs_service.ingest_triples([
                    {
                        "subject": agent_role,
                        "subject_type": "agent",
                        "predicate": "USED_SKILL",
                        "object": skill_name,
                        "object_type": "skill",
                        "timestamp": time.time()
                    }
                ])
            except Exception:
                pass

        # Push synapse weight adjustment event to spatial broadcaster
        if self._spatial_broadcaster:
            try:
                self._spatial_broadcaster.push_event({
                    "event_type": "ternary_synapse_adjustment",
                    "role": agent_role,
                    "skill_name": skill_name,
                    "success": success,
                    "weight_adjustment": 1 if success else -1,
                    "timestamp": time.time()
                })
            except Exception:
                pass

    def get_matrix(self) -> dict:
        return {
            "proficiency_matrix": {role: dict(skills) for role, skills in self.proficiency_matrix.items()},
            "skill_events": self._skill_events[-20:],
            "agent_count": len(self.proficiency_matrix),
            "skill_count": len(set(s for m in self.proficiency_matrix.values() for s in m)),
            "timestamp": datetime.now().isoformat()
        }

    def get_topology_overlay(self, agent_health_scores: dict = None) -> dict:
        """Provide layout-ready nodes and link weights overlay for D3 force layouts."""
        from swarm_v2.core.agent_mesh import get_agent_mesh
        mesh = get_agent_mesh()
        
        nodes = []
        for role, skills in self.proficiency_matrix.items():
            top_skill = max(skills, key=skills.get) if skills else None
            top_prof = skills.get(top_skill, 0.0) if top_skill else 0.0
            
            task_count = 0
            if mesh:
                for node in mesh.nodes.values():
                    if node.role == role:
                        task_count = node.task_count
                        break

            nodes.append({
                "id": role,
                "skills": [{"name": k, "proficiency": v} for k, v in skills.items()],
                "top_skill": top_skill,
                "top_proficiency": top_prof,
                "health_score": agent_health_scores.get(role, 1.0) if agent_health_scores else 1.0,
                "task_count": task_count
            })

        # Calculate skill-sharing affinity/co-links
        skill_links = []
        roles = list(self.proficiency_matrix.keys())
        for i, role_a in enumerate(roles):
            for role_b in roles[i+1:]:
                skills_a = set(self.proficiency_matrix[role_a].keys())
                skills_b = set(self.proficiency_matrix[role_b].keys())
                shared = skills_a & skills_b
                if shared:
                    # Sum of minimum proficiency of shared skills
                    affinity = sum(min(self.proficiency_matrix[role_a][s], self.proficiency_matrix[role_b][s]) for s in shared)
                    skill_links.append({
                        "source": role_a,
                        "target": role_b,
                        "shared_skills": list(shared),
                        "affinity": round(affinity, 3)
                    })

        return {
            "nodes": nodes,
            "skill_links": skill_links,
            "timestamp": datetime.now().isoformat()
        }

_instance = None

def get_skill_matrix_sync() -> SkillMatrixSync:
    global _instance
    if _instance is None:
        _instance = SkillMatrixSync()
    return _instance
