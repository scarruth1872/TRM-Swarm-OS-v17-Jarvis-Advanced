"""
High-Bandwidth Telemetry Fabric (HBTF) Module — Phase 2: Platform Integration
Provides a dedicated, low-latency, high-throughput pub-sub event fabric
for direct ingestion of raw microsecond telemetry data from Swarm OS nodes.
"""

import asyncio
import logging
import time
from typing import Dict, List, Callable, Any, Optional
from pydantic import BaseModel, Field

logger = logging.getLogger("TelemetryFabric")


class TelemetryFrame(BaseModel):
    """Raw telemetry frame emitted by node sensors."""
    frame_id: str
    source_node: str
    channel: str  # "metrics", "security_events", "topology", "ipc_stream"
    payload: Dict[str, Any]
    timestamp: float = Field(default_factory=time.time)


class TelemetryFabric:
    """
    High-Bandwidth Telemetry Fabric (HBTF).
    Supports non-blocking async pub-sub topic routing across internal modules.
    """

    def __init__(self):
        self.subscribers: Dict[str, List[Callable[[TelemetryFrame], None]]] = {}
        self.frame_buffer: List[TelemetryFrame] = []
        self.max_buffer_size: int = 1000
        self.total_frames_processed: int = 0

    def subscribe(self, channel: str, callback: Callable[[TelemetryFrame], None]):
        """Subscribe a module callback to a telemetry channel."""
        if channel not in self.subscribers:
            self.subscribers[channel] = []
        self.subscribers[channel].append(callback)
        logger.debug(f"[HBTF] New subscriber registered on channel '{channel}'")

    def publish(self, frame: TelemetryFrame):
        """Publish a raw telemetry frame to all channel subscribers."""
        self.total_frames_processed += 1

        # Buffer management
        if len(self.frame_buffer) >= self.max_buffer_size:
            self.frame_buffer.pop(0)
        self.frame_buffer.append(frame)

        # Notify subscribers
        callbacks = self.subscribers.get(frame.channel, [])
        for cb in callbacks:
            try:
                cb(frame)
            except Exception as e:
                logger.error(f"[HBTF Error] Subscriber callback failed for frame {frame.frame_id}: {e}")

        # Also push to Semantic Knowledge Graph
        try:
            from swarm_v2.core.semantic_knowledge_graph import get_semantic_knowledge_graph
            skg = get_semantic_knowledge_graph()
            skg.ingest_telemetry_event({
                "node_id": frame.source_node,
                "properties": frame.payload,
                "health_score": frame.payload.get("health_score", 1.0)
            })
        except Exception:
            pass

    def get_fabric_metrics(self) -> Dict[str, Any]:
        """Return operational metrics for the Telemetry Fabric."""
        return {
            "total_frames_processed": self.total_frames_processed,
            "buffered_frames": len(self.frame_buffer),
            "active_channels": list(self.subscribers.keys()),
            "status": "active"
        }


_telemetry_fabric: Optional[TelemetryFabric] = None


def get_telemetry_fabric() -> TelemetryFabric:
    global _telemetry_fabric
    if _telemetry_fabric is None:
        _telemetry_fabric = TelemetryFabric()
    return _telemetry_fabric
