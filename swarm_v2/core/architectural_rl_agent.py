"""
Architectural RL Agent Module — Phase 1: Cognitive Foundation
Explores and optimizes architectural decision spaces (resource allocation, microservice decomposition,
token rate limits) based on reward functions for latency, throughput, cost, and security.
"""

import logging
import random
import time
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("ArchitecturalRLAgent")


class PolicyState(BaseModel):
    """Current environmental state vector for the RL Policy Agent."""
    cpu_percent: float = 20.0
    memory_percent: float = 40.0
    active_nodes: int = 12
    avg_latency_ms: float = 15.0
    error_rate: float = 0.01


class ArchitecturalAction(BaseModel):
    """Action chosen by the RL Policy Agent."""
    action_type: str  # "scale_nodes", "adjust_token_rate", "rebalance_memory", "optimize_routing"
    parameters: Dict[str, Any] = Field(default_factory=dict)
    expected_reward: float = 0.0


class ArchitecturalRLAgent:
    """
    Reinforcement Learning policy agent that optimizes Swarm OS parameters
    by evaluating reward tradeoffs across latency, cost, and resilience.
    """

    def __init__(self):
        self.history: List[Dict[str, Any]] = []
        self.current_reward: float = 0.0
        self.learning_rate: float = 0.1
        self.discount_factor: float = 0.9

    def evaluate_reward(self, state: PolicyState, action: ArchitecturalAction) -> float:
        """
        Multi-objective reward function:
        Reward = + Throughput/Efficiency - (0.4 * Latency) - (0.5 * Resource_Spike) - (1.0 * Error_Rate)
        """
        latency_penalty = min(state.avg_latency_ms / 100.0, 2.0)
        cpu_penalty = max(0.0, (state.cpu_percent - 80.0) / 20.0)
        error_penalty = state.error_rate * 10.0

        base_reward = 1.0
        reward = base_reward - (0.4 * latency_penalty) - (0.5 * cpu_penalty) - (1.0 * error_penalty)

        # Action specific bonuses
        if action.action_type == "adjust_token_rate" and state.cpu_percent > 70:
            reward += 0.5
        elif action.action_type == "optimize_routing" and state.avg_latency_ms > 50:
            reward += 0.6

        return round(reward, 3)

    def select_action(self, state: PolicyState) -> ArchitecturalAction:
        """Epsilon-greedy policy selection for system parameter optimization."""
        candidate_actions = [
            ArchitecturalAction(action_type="adjust_token_rate", parameters={"token_limit": 200}),
            ArchitecturalAction(action_type="optimize_routing", parameters={"algorithm": "qiae_quantum"}),
            ArchitecturalAction(action_type="rebalance_memory", parameters={"gc_threshold": 0.85}),
            ArchitecturalAction(action_type="scale_nodes", parameters={"target_capacity": state.active_nodes + 1}),
        ]

        best_action = candidate_actions[0]
        best_reward = -999.0

        for action in candidate_actions:
            r = self.evaluate_reward(state, action)
            action.expected_reward = r
            if r > best_reward:
                best_reward = r
                best_action = action

        # Save to history
        self.current_reward = best_reward
        self.history.append({
            "timestamp": time.time(),
            "state": state.model_dump(),
            "action": best_action.model_dump(),
            "reward": best_reward
        })

        logger.info(f"[RL Agent] Selected action '{best_action.action_type}' with expected reward: {best_reward}")
        return best_action


_rl_agent: Optional[ArchitecturalRLAgent] = None


def get_architectural_rl_agent() -> ArchitecturalRLAgent:
    global _rl_agent
    if _rl_agent is None:
        _rl_agent = ArchitecturalRLAgent()
    return _rl_agent
