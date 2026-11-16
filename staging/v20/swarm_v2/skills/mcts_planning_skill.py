# TRM Swarm OS — Monte Carlo Tree Search Planning Skill
# Absorbed from Agent-Q & Quantum-Inspired Agent Ecosystem Architecture
# Monad Anchor: 0x4D4F4E4144000001

import math
import random
import time
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional

@dataclass
class MCTSNode:
    state_description: str
    action_taken: Optional[str] = None
    parent: Optional['MCTSNode'] = None
    children: List['MCTSNode'] = field(default_factory=list)
    visits: int = 0
    total_reward: float = 0.0
    k3_trit: int = 0  # -1 (False), 0 (Unknown), +1 (True)
    metadata: Dict[str, Any] = field(default_factory=dict)

    @property
    def ucb1_score(self, exploration_weight: float = 1.414) -> float:
        if self.visits == 0:
            return float('inf')
        exploitation = self.total_reward / self.visits
        parent_visits = self.parent.visits if self.parent else self.visits
        exploration = exploration_weight * math.sqrt(math.log(max(1, parent_visits)) / self.visits)
        return exploitation + exploration

class MCTSPlanner:
    def __init__(self, max_iterations: int = 64, max_depth: int = 8, exploration_weight: float = 1.414):
        self.max_iterations = max_iterations
        self.max_depth = max_depth
        self.exploration_weight = exploration_weight

    def plan(self, initial_problem: str, candidate_actions_fn: Any = None, evaluate_state_fn: Any = None) -> Dict[str, Any]:
        t0 = time.perf_counter()
        root = MCTSNode(state_description=initial_problem)

        default_actions = [
            'Deconstruct goal into invariant axioms',
            'Probe active skill matrix & agent registries',
            'Cross-validate hypotheses against SKG graph',
            'Simulate atomic step in isolated sandbox',
            'Synthesize multi-agent consensus attestation'
        ]

        actions_provider = candidate_actions_fn or (lambda state: default_actions)
        evaluator = evaluate_state_fn or self._default_evaluator

        for _ in range(self.max_iterations):
            leaf, depth = self._select(root)
            if depth < self.max_depth and not leaf.children:
                actions = actions_provider(leaf.state_description)
                self._expand(leaf, actions)
            
            node_to_eval = random.choice(leaf.children) if leaf.children else leaf
            reward = evaluator(node_to_eval.state_description)
            self._backpropagate(node_to_eval, reward)

        best_child = max(root.children, key=lambda c: c.visits) if root.children else root
        elapsed_ms = (time.perf_counter() - t0) * 1000

        return {
            'status': 'completed',
            'problem': initial_problem,
            'root_visits': root.visits,
            'best_action': best_child.action_taken,
            'best_reward_mean': (best_child.total_reward / max(1, best_child.visits)) if best_child.visits else 0.0,
            'tree_depth': self._get_max_depth(root),
            'total_nodes': self._count_nodes(root),
            'execution_time_ms': round(elapsed_ms, 2),
            'iterations': self.max_iterations
        }

    def _select(self, node: MCTSNode) -> (MCTSNode, int):
        current = node
        depth = 0
        while current.children:
            unvisited = [c for c in current.children if c.visits == 0]
            if unvisited:
                return random.choice(unvisited), depth + 1
            current = max(current.children, key=lambda c: c.ucb1_score)
            depth += 1
        return current, depth

    def _expand(self, parent: MCTSNode, actions: List[str]):
        for act in actions:
            child_state = f'{parent.state_description} -> Step: {act}'
            child = MCTSNode(state_description=child_state, action_taken=act, parent=parent)
            parent.children.append(child)

    def _default_evaluator(self, state: str) -> float:
        score = 0.5
        if 'consensus' in state.lower(): score += 0.2
        if 'axioms' in state.lower(): score += 0.15
        if 'isolated' in state.lower(): score += 0.1
        score += random.uniform(-0.05, 0.05)
        return max(0.0, min(1.0, score))

    def _backpropagate(self, node: Optional[MCTSNode], reward: float):
        curr = node
        while curr is not None:
            curr.visits += 1
            curr.total_reward += reward
            curr = curr.parent

    def _get_max_depth(self, node: MCTSNode) -> int:
        if not node.children: return 1
        return 1 + max(self._get_max_depth(c) for c in node.children)

    def _count_nodes(self, node: MCTSNode) -> int:
        return 1 + sum(self._count_nodes(c) for c in node.children)

_planner = None
def get_mcts_planner() -> MCTSPlanner:
    global _planner
    if _planner is None:
        _planner = MCTSPlanner()
    return _planner
