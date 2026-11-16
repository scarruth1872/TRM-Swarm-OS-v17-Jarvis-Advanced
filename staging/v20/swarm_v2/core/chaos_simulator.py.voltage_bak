"""
Chaos Mesh Simulator — Phase 15: Cross-Reality Coherence
Injects virtual latencies, partitions, and node dropout states to stress-test consensus layers.
"""

import time
import logging
from typing import Dict, List, Any
from pydantic import BaseModel

logger = logging.getLogger("ChaosSimulator")

class ChaosConfig(BaseModel):
    simulate_latency_ms: int = 0
    offline_nodes: List[str] = []
    inject_duplicate_txs: bool = False

class ChaosSimulator:
    """Dynamic fault injection harness for checking P2P edge mesh self-healing strategies."""
    
    def __init__(self):
        self.active_session = False
        self.config = ChaosConfig()
        logger.info("[ChaosSimulator] Chaos Mesh Engine loaded.")

    def start_session(self, config: ChaosConfig) -> bool:
        """Launches simulated chaos conditions with targeted fault profiles."""
        self.config = config
        self.active_session = True
        logger.warning(
            f"[ChaosSimulator] Chaos Session STARTED. Latency: {config.simulate_latency_ms}ms, "
            f"Offline Nodes: {config.offline_nodes}, Duplicate Injection: {config.inject_duplicate_txs}"
        )
        return True

    def stop_session(self) -> bool:
        """Cleans and tears down all injected system anomalies."""
        self.active_session = False
        self.config = ChaosConfig()
        logger.info("[ChaosSimulator] Chaos Session STOPPED. All systems restored to normal.")
        return True

    def process_latency(self) -> float:
        """Virtual delay factor returned based on current chaos configurations."""
        if self.active_session and self.config.simulate_latency_ms > 0:
            return self.config.simulate_latency_ms / 1000.0
        return 0.0

    def is_node_isolated(self, node_id: str) -> bool:
        """Determines if a node is virtually segmented or marked offline in the active simulation."""
        if self.active_session:
            return node_id in self.config.offline_nodes
        return False

# Singleton
_chaos_simulator = None

def get_chaos_simulator() -> ChaosSimulator:
    global _chaos_simulator
    if _chaos_simulator is None:
        _chaos_simulator = ChaosSimulator()
    return _chaos_simulator
