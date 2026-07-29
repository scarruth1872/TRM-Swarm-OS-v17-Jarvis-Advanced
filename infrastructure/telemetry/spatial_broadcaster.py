"""
Phase 12: Spatial Telemetry Broadcaster
Listens to Phase 11 Quantum Entanglement and Biomimetic Healing events
and compiles them into a unified, queryable stream for the frontend dashboard.
"""

import time
import logging
from typing import List, Dict, Any
from collections import deque

from infrastructure.routing.qer_mesh import get_qer_mesh

logger = logging.getLogger("SpatialBroadcaster")

class SpatialTelemetryDaemon:
    def __init__(self):
        self.qer = get_qer_mesh()
        
        # Ring buffers to hold the latest telemetry events without memory leaking
        self.entanglement_log: deque = deque(maxlen=500)
        self.mitosis_log: deque = deque(maxlen=200)
        
        logger.info("Spatial Telemetry Daemon initialized. Awaiting Phase 11 events.")
        
        # We hook into a specific global QER state designated for telemetry
        # In a real system, we'd iterate over all active entangled states, but for Phase 12,
        # we will register an observer on a master telemetry channel.
        self._master_telemetry = self.qer.get_or_create_entanglement("master_telemetry_stream")
        self._master_telemetry.register_observer(self._on_qer_event)

    def _on_qer_event(self, state_id: str, key: str, value: Any):
        """Callback triggered instantaneously when QER state changes."""
        event = {
            "timestamp": time.time(),
            "state_id": state_id,
            "key": key,
            "value": str(value)
        }
        self.entanglement_log.appendleft(event)
        
    def log_mitosis_event(self, parent_id: str, child_id: str, trigger_stress: float):
        """Called by the Biomimetic Matrix when cellular division occurs."""
        event = {
            "timestamp": time.time(),
            "event_type": "mitosis",
            "parent_agent_id": parent_id,
            "child_agent_id": child_id,
            "stress_threshold_exceeded": trigger_stress
        }
        self.mitosis_log.appendleft(event)
        logger.debug(f"[Broadcaster] Mitosis logged: {parent_id} -> {child_id}")

    def get_recent_entanglements(self, count: int = 50) -> List[Dict[str, Any]]:
        return list(self.entanglement_log)[:count]
        
    def get_recent_mitosis(self, count: int = 50) -> List[Dict[str, Any]]:
        return list(self.mitosis_log)[:count]

# Singleton accessor
_broadcaster_instance = None
def get_spatial_broadcaster() -> SpatialTelemetryDaemon:
    global _broadcaster_instance
    if _broadcaster_instance is None:
        _broadcaster_instance = SpatialTelemetryDaemon()
    return _broadcaster_instance
