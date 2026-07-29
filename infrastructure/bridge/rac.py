"""
Swarm OS v12 - Reality-Aware Consensus (RAC)
Phase 10 component for cross-reality orchestration.
Weights node influence in consensus based on their cross-dimensional bridging density.
"""

import logging
from typing import Dict, List

logger = logging.getLogger("RAC")

class RealityNode:
    def __init__(self, node_id: str, connected_realities: List[str]):
        self.node_id = node_id
        self.connected_realities = connected_realities
        
    @property
    def density(self) -> int:
        """
        Density is the number of distinct realities this node bridges.
        """
        return len(set(self.connected_realities))

class RealityAwareConsensus:
    def __init__(self):
        self.nodes: Dict[str, RealityNode] = {}
        logger.info("[RAC] Initialized Reality-Aware Consensus engine.")

    def register_node(self, node_id: str, connected_realities: List[str]):
        self.nodes[node_id] = RealityNode(node_id, connected_realities)
        logger.debug(f"[RAC] Registered node {node_id} bridging {len(connected_realities)} realities.")

    def calculate_voting_weight(self, node_id: str) -> float:
        """
        Calculates the consensus voting power of a node.
        Base weight is 1.0, plus 0.5 for each additional reality bridged beyond the first.
        """
        if node_id not in self.nodes:
            return 0.0
            
        node = self.nodes[node_id]
        if node.density == 0:
            return 0.0
            
        weight = 1.0 + (0.5 * (node.density - 1))
        return weight

    def run_consensus(self, votes: Dict[str, str]) -> str:
        """
        Runs a weighted vote to determine consensus across the bridged network.
        votes mapping: node_id -> voted_proposal
        """
        logger.info("[RAC] Running cross-dimensional consensus...")
        
        tally: Dict[str, float] = {}
        for node_id, proposal in votes.items():
            weight = self.calculate_voting_weight(node_id)
            if weight > 0:
                tally[proposal] = tally.get(proposal, 0.0) + weight
                logger.debug(f"[RAC] Node {node_id} (weight: {weight:.1f}) voted for {proposal}")
                
        if not tally:
            logger.warning("[RAC] Consensus failed: No valid votes.")
            return "NO_CONSENSUS"
            
        # Determine winner
        winner = max(tally, key=tally.get)
        total_weight = sum(tally.values())
        winning_weight = tally[winner]
        
        logger.info(f"[RAC] Consensus Reached! Proposal '{winner}' won with {winning_weight:.1f}/{total_weight:.1f} weighted votes.")
        return winner

# Singleton
_rac = RealityAwareConsensus()

def get_rac() -> RealityAwareConsensus:
    return _rac
