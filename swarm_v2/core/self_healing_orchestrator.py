import asyncio
import time
import logging
from collections import deque
from contextlib import asynccontextmanager
from typing import Dict, List, Optional, Any
from datetime import datetime

logger = logging.getLogger(__name__)

class SelfHealingOrchestrator:
    """
    Self-Healing Orchestrator for TRM Swarm OS.
    Instruments task execution latency and error rates, triggering role reassignment
    when nodes degrade, and restoring them when they recover.
    """
    HEALTH_THRESHOLD = 0.35
    LATENCY_CEILING = 8.0  # Max acceptable task latency in seconds
    RECOVERY_STREAK_REQUIRED = 5
    WINDOW_SIZE = 50

    def __init__(self):
        self._latency_windows: Dict[str, deque] = {}  # agent_role -> deque
        self._error_counts: Dict[str, int] = {}
        self._task_counts: Dict[str, int] = {}
        self._cpu_loads: Dict[str, float] = {}
        self._degraded_agents: Dict[str, dict] = {}  # role -> details
        self._reassignment_log: List[dict] = []
        self._active: bool = False
        self._agent_mesh = None

    def bind_mesh(self, agent_mesh):
        """Bind the active AgentMesh instance."""
        self._agent_mesh = agent_mesh

    @asynccontextmanager
    async def track_task(self, agent_role: str):
        """
        Async context manager to monitor agent latency and task outcomes.
        """
        start_time = time.monotonic()
        if agent_role not in self._latency_windows:
            self._latency_windows[agent_role] = deque(maxlen=self.WINDOW_SIZE)
        if agent_role not in self._task_counts:
            self._task_counts[agent_role] = 0
        if agent_role not in self._error_counts:
            self._error_counts[agent_role] = 0

        try:
            yield
            elapsed = time.monotonic() - start_time
            self._latency_windows[agent_role].append(elapsed)
            self._task_counts[agent_role] += 1

            # Handle recovery logic if the agent is currently degraded
            if agent_role in self._degraded_agents:
                details = self._degraded_agents[agent_role]
                if elapsed < self.LATENCY_CEILING * 0.6:  # Healthy threshold (4.8s)
                    details["recovery_streak"] += 1
                    if details["recovery_streak"] >= self.RECOVERY_STREAK_REQUIRED:
                        self._restore_agent(agent_role)
                else:
                    details["recovery_streak"] = 0

        except Exception as e:
            elapsed = time.monotonic() - start_time
            self._latency_windows[agent_role].append(elapsed)
            self._task_counts[agent_role] += 1
            self._error_counts[agent_role] += 1
            logger.error(f"[SelfHealing] Task for {agent_role} failed with error: {e}")
            raise e

    def compute_health_score(self, agent_role: str) -> float:
        """Calculate composite health score between 0.0 and 1.0."""
        window = self._latency_windows.get(agent_role)
        if not window:
            avg_latency = 0.5
        else:
            avg_latency = sum(window) / max(len(window), 1)

        normalized_latency = min(avg_latency / self.LATENCY_CEILING, 1.0)
        
        task_count = self._task_counts.get(agent_role, 0)
        error_count = self._error_counts.get(agent_role, 0)
        error_rate = error_count / max(task_count, 1)

        cpu_load = self._cpu_loads.get(agent_role, 0.3)

        health = (1.0 - normalized_latency) * 0.4 + (1.0 - error_rate) * 0.3 + (1.0 - cpu_load) * 0.3
        return max(0.0, min(1.0, health))

    def update_cpu_load(self, agent_role: str, cpu_load: float):
        """Update last known CPU load (0.0 to 1.0)."""
        self._cpu_loads[agent_role] = cpu_load

    def _evaluate_all_agents(self) -> List[str]:
        """Identify agents whose health has dropped below the threshold."""
        degraded = []
        for role in self._latency_windows:
            if self.compute_health_score(role) < self.HEALTH_THRESHOLD:
                degraded.append(role)
        return degraded

    def _reassign_role(self, degraded_role: str, engine_team: dict):
        """Transfer specialties from degraded agent to healthiest active peer."""
        if not self._agent_mesh:
            logger.warning("[SelfHealing] Cannot reassign: Mesh not bound.")
            return

        # Find degraded node
        degraded_node = None
        for node in self._agent_mesh.nodes.values():
            if node.role == degraded_role:
                degraded_node = node
                break

        if not degraded_node:
            logger.warning(f"[SelfHealing] Degraded agent node for role {degraded_role} not found in mesh.")
            return

        # Find healthiest candidate that is not degraded
        best_candidate = None
        best_score = -1.0

        for role in engine_team:
            if role == degraded_role or role in self._degraded_agents:
                continue
            
            score = self.compute_health_score(role)
            if score > best_score:
                best_score = score
                best_candidate = role

        if not best_candidate:
            logger.error("[SelfHealing] No healthy candidate available for role reassignment.")
            return

        # Find recipient node
        recipient_node = None
        for node in self._agent_mesh.nodes.values():
            if node.role == best_candidate:
                recipient_node = node
                break

        if not recipient_node:
            logger.error(f"[SelfHealing] Recipient node for role {best_candidate} not found.")
            return

        # Transfer specialties
        original_specs = list(degraded_node.specialties)
        added_specs = []
        for spec in original_specs:
            if spec not in recipient_node.specialties:
                recipient_node.specialties.append(spec)
                added_specs.append(spec)

        degraded_node.status = "degraded"
        
        # Save details for healing/recovery
        self._degraded_agents[degraded_role] = {
            "degraded_at": datetime.now().isoformat(),
            "original_specialties": original_specs,
            "added_specialties": added_specs,
            "transferred_to": best_candidate,
            "recovery_streak": 0
        }

        event = {
            "type": "self_healing_reassignment",
            "degraded_role": degraded_role,
            "assigned_to": best_candidate,
            "transferred_specialties": added_specs,
            "timestamp": datetime.now().isoformat()
        }
        self._reassignment_log.append(event)
        self._agent_mesh.topology_version += 1
        self._agent_mesh._save_state()

        logger.warning(f"[SelfHealing] Role Reassigned: transferred '{degraded_role}' specialties {added_specs} to '{best_candidate}' due to degradation.")

    def _restore_agent(self, agent_role: str):
        """Restore originally assigned specialties to recovered agent."""
        if agent_role not in self._degraded_agents:
            return

        details = self._degraded_agents[agent_role]
        transferred_to = details["transferred_to"]
        added_specs = details["added_specialties"]

        # Find nodes
        degraded_node = None
        recipient_node = None
        if self._agent_mesh:
            for node in self._agent_mesh.nodes.values():
                if node.role == agent_role:
                    degraded_node = node
                elif node.role == transferred_to:
                    recipient_node = node

        if degraded_node:
            degraded_node.status = "online"
        
        if recipient_node:
            # Revoke specialties transferred during degradation
            for spec in added_specs:
                if spec in recipient_node.specialties:
                    recipient_node.specialties.remove(spec)

        del self._degraded_agents[agent_role]

        event = {
            "type": "self_healing_restoration",
            "restored_role": agent_role,
            "revoked_from": transferred_to,
            "timestamp": datetime.now().isoformat()
        }
        self._reassignment_log.append(event)

        if self._agent_mesh:
            self._agent_mesh.topology_version += 1
            self._agent_mesh._save_state()

        logger.info(f"[SelfHealing] Role Restored: '{agent_role}' recovered stability. Revoked transient specialties from '{transferred_to}'.")

    async def _healing_loop(self, agent_mesh, engine_team: dict):
        """Autonomic background loop monitoring agent nodes health."""
        self._active = True
        self.bind_mesh(agent_mesh)
        logger.info("[SelfHealing] Loop activated.")
        
        while self._active:
            try:
                await asyncio.sleep(20)
                degraded_roles = self._evaluate_all_agents()
                for role in degraded_roles:
                    if role not in self._degraded_agents:
                        self._reassign_role(role, engine_team)
            except Exception as e:
                logger.error(f"[SelfHealing] Error in autonomic loop: {e}")

    async def stop(self):
        self._active = False

    def get_status(self) -> dict:
        return {
            "agent_health_scores": {role: round(self.compute_health_score(role), 3) for role in self._latency_windows},
            "degraded_agents": dict(self._degraded_agents),
            "recent_reassignments": self._reassignment_log[-10:],
            "healing_active": self._active,
            "total_tasks_tracked": sum(self._task_counts.values())
        }

    def force_reassign(self, from_role: str, to_role: str, engine_team: dict) -> dict:
        """Manually force reassignment of specialties."""
        if not self._agent_mesh:
            raise ValueError("Mesh not bound")

        degraded_node = None
        recipient_node = None
        for node in self._agent_mesh.nodes.values():
            if node.role == from_role:
                degraded_node = node
            elif node.role == to_role:
                recipient_node = node

        if not degraded_node or not recipient_node:
            raise ValueError("Nodes not found")

        original_specs = list(degraded_node.specialties)
        added_specs = []
        for spec in original_specs:
            if spec not in recipient_node.specialties:
                recipient_node.specialties.append(spec)
                added_specs.append(spec)

        degraded_node.status = "degraded"
        
        self._degraded_agents[from_role] = {
            "degraded_at": datetime.now().isoformat(),
            "original_specialties": original_specs,
            "added_specialties": added_specs,
            "transferred_to": to_role,
            "recovery_streak": 0
        }

        event = {
            "type": "force_reassignment",
            "degraded_role": from_role,
            "assigned_to": to_role,
            "transferred_specialties": added_specs,
            "timestamp": datetime.now().isoformat()
        }
        self._reassignment_log.append(event)
        self._agent_mesh.topology_version += 1
        self._agent_mesh._save_state()

        return self.get_status()

_instance = None

def get_self_healing_orchestrator() -> SelfHealingOrchestrator:
    global _instance
    if _instance is None:
        _instance = SelfHealingOrchestrator()
    return _instance
