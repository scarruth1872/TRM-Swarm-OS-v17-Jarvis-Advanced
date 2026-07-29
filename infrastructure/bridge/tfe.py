"""
Swarm OS v12 - Temporal Folding Engine (TFE)
Phase 10 component for cross-reality orchestration.
Implements multi-dimensional Lamport logical clocks to synchronize cross-domain events.
"""

import logging
from typing import Dict, List

logger = logging.getLogger("TFE")

class MultiDimensionalClock:
    def __init__(self, node_id: str):
        self.node_id = node_id
        # Vector clock: maps reality_id -> logical_time
        self.vector: Dict[str, int] = {node_id: 0}

    def increment(self):
        self.vector[self.node_id] += 1

    def merge(self, incoming_vector: Dict[str, int]):
        """
        Updates the local vector clock based on an incoming message's vector.
        """
        for r_id, time_val in incoming_vector.items():
            current_val = self.vector.get(r_id, 0)
            self.vector[r_id] = max(current_val, time_val)
        # Always increment own time after processing an event
        self.increment()

class TemporalFoldingEngine:
    def __init__(self):
        self.clocks: Dict[str, MultiDimensionalClock] = {}
        logger.info("[TFE] Initialized Temporal Folding Engine with Vector Clocks.")

    def register_node(self, node_id: str):
        if node_id not in self.clocks:
            self.clocks[node_id] = MultiDimensionalClock(node_id)
            logger.debug(f"[TFE] Registered node {node_id} to temporal engine.")

    def send_event(self, sender_id: str) -> Dict[str, int]:
        """
        Simulates sending an event across realities.
        Returns the sender's current vector clock.
        """
        if sender_id not in self.clocks:
            self.register_node(sender_id)
            
        clock = self.clocks[sender_id]
        clock.increment()
        
        logger.debug(f"[TFE] Node {sender_id} emitted event at logical time {clock.vector}")
        return dict(clock.vector) # Return copy

    def receive_event(self, receiver_id: str, sender_vector: Dict[str, int]):
        """
        Simulates receiving an event from another reality and synchronizing time.
        """
        if receiver_id not in self.clocks:
            self.register_node(receiver_id)
            
        clock = self.clocks[receiver_id]
        clock.merge(sender_vector)
        
        logger.debug(f"[TFE] Node {receiver_id} synchronized time. New logical time {clock.vector}")

# Singleton
_tfe = TemporalFoldingEngine()

def get_tfe() -> TemporalFoldingEngine:
    return _tfe
