"""
Phase 7: Cognitive Load Balancer
Dynamically distributes reasoning tasks across the swarm based on
per-agent load, specialization, and historical performance.
"""

import logging
import time
from collections import defaultdict
from typing import Any, Dict, List, Optional

logger = logging.getLogger("CognitiveLoadBalancer")


class CognitiveLoadBalancer:
    """Tracks per-agent load and redistributes tasks to the least-loaded capable agent."""

    def __init__(self):
        # role -> { task_count, avg_completion_time, current_complexity, last_active }
        self._agent_load: Dict[str, Dict[str, float]] = {}
        self._lock = False  # simple flag for thread safety hint
        self._capabilities: Dict[str, List[str]] = {
            "Logic": ["reasoning", "analysis", "math", "logic"],
            "Archi": ["architecture", "design", "planning", "review"],
            "Devo": ["implementation", "code", "development", "debugging"],
            "Shield": ["security", "audit", "validation", "compliance"],
            "Seeker": ["research", "discovery", "investigation", "intel"],
            "Bridge": ["integration", "api", "synthesis", "connection"],
            "Flow": ["pipeline", "automation", "devops", "deployment"],
            "Scribe": ["documentation", "writing", "reporting"],
            "Verify": ["testing", "qa", "verification", "assertion"],
            "Pulse": ["monitoring", "health", "telemetry", "alerts"],
            "Vision": ["ui", "design", "ux", "frontend"],
            "Orchestra": ["management", "coordination", "oversight"],
        }
        self._completion_history: Dict[str, List[float]] = defaultdict(list)
        self._max_history = 50

    def record_task_start(self, agent_role: str) -> None:
        """Record that an agent started a task."""
        now = time.time()
        if agent_role not in self._agent_load:
            self._agent_load[agent_role] = {
                "task_count": 0,
                "avg_completion_time": 0.0,
                "current_complexity": 0.0,
                "last_active": now,
            }
        self._agent_load[agent_role]["task_count"] += 1
        self._agent_load[agent_role]["last_active"] = now
        logger.debug(f"[Load] {agent_role} started task. Count: {self._agent_load[agent_role]['task_count']}")

    def record_task_end(self, agent_role: str, completion_time: float, complexity: float = 0.0) -> None:
        """Record task completion and update moving average."""
        if agent_role not in self._agent_load:
            return

        self._agent_load[agent_role]["current_complexity"] = max(0.0, complexity)
        self._agent_load[agent_role]["last_active"] = time.time()
        self._agent_load[agent_role]["task_count"] = max(0, self._agent_load[agent_role]["task_count"] - 1)

        # Track completion history for moving average
        history = self._completion_history[agent_role]
        history.append(completion_time)
        if len(history) > self._max_history:
            history.pop(0)
        if history:
            self._agent_load[agent_role]["avg_completion_time"] = sum(history) / len(history)

        logger.debug(f"[Load] {agent_role} completed in {completion_time:.2f}s")

    def get_agent_load(self, agent_role: str) -> Dict[str, Any]:
        """Get current load metrics for a specific agent."""
        default = {"task_count": 0, "avg_completion_time": 0.0, "current_complexity": 0.0, "last_active": 0.0}
        return {**default, **self._agent_load.get(agent_role, default)}

    def get_swarm_load_summary(self) -> Dict[str, Any]:
        """Get load distribution across all agents."""
        summary = {}
        for role in self._capabilities:
            summary[role] = self.get_agent_load(role)

        total_tasks = sum(v["task_count"] for v in summary.values())
        active_agents = sum(1 for v in summary.values() if v["task_count"] > 0)

        return {
            "agents": summary,
            "total_pending_tasks": total_tasks,
            "active_agents": active_agents,
            "total_agents": len(self._capabilities),
            "utilization_pct": round((active_agents / len(self._capabilities)) * 100, 1) if self._capabilities else 0,
            "timestamp": time.time(),
        }

    def redistribute_task(self, task_description: str, preferred_roles: List[str]) -> Optional[str]:
        """
        Select the least-loaded agent among preferred roles.
        Returns the agent role name, or None if none available.
        """
        if not preferred_roles:
            preferred_roles = list(self._capabilities.keys())

        candidates = []
        for role in preferred_roles:
            if role not in self._capabilities:
                continue
            load = self.get_agent_load(role)
            # Score: lower task_count = better, adjust for capability match
            score = load["task_count"] + (load["current_complexity"] * 0.5)
            candidates.append((score, role))

        if not candidates:
            return None

        candidates.sort(key=lambda x: x[0])
        selected = candidates[0][1]
        logger.info(f"[Load] Redistributed task to {selected} (load score: {candidates[0][0]:.1f})")
        return selected

    def get_capable_roles(self, capability: str) -> List[str]:
        """Return all agent roles that have a specific capability."""
        return [role for role, caps in self._capabilities.items() if capability in caps]


# Singleton
_load_balancer: Optional[CognitiveLoadBalancer] = None


def get_load_balancer() -> CognitiveLoadBalancer:
    global _load_balancer
    if _load_balancer is None:
        _load_balancer = CognitiveLoadBalancer()
    return _load_balancer
