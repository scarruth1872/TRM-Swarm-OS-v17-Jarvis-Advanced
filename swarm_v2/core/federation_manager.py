"""
Federation Manager — Phase 15: Collective Singularity
High-level orchestrator for multi-swarm connectivity, health monitoring, and peer discovery.
"""

import asyncio
import logging
import time
from typing import List, Dict, Optional
from datetime import datetime
from swarm_v2.core.federation import get_federation, SwarmNode

logger = logging.getLogger(__name__)

class FederationManager:
    """
    Manages the lifecycle of connections between Swarm OS instances.
    Provides proactive peer discovery and mesh health analytics.
    """
    
    def __init__(self, check_interval: int = 60):
        self.check_interval = check_interval
        self.federation = get_federation()
        self.is_running = False
        self._background_tasks = []
        
    async def start(self):
        """Start proactive federation management."""
        if not self.federation:
            logger.warning("[FederationManager] Federation protocol not initialized. Manager standby.")
            return
            
        self.is_running = True
        logger.info("[FederationManager] Collective Singularity Manager online.")
        
        # Start background loops
        self._background_tasks.append(asyncio.create_task(self._discovery_loop()))
        self._background_tasks.append(asyncio.create_task(self._health_analytics_loop()))

    async def stop(self):
        """Shutdown the manager."""
        self.is_running = False
        for task in self._background_tasks:
            task.cancel()
        logger.info("[FederationManager] Manager offline.")

    async def _discovery_loop(self):
        """Proactively discover new peers from existing connections."""
        while self.is_running:
            try:
                online_nodes = self.federation.registry.get_online_nodes()
                for node in online_nodes:
                    logger.debug(f"[FederationManager] Probing {node.name} for new peers...")
                    new_peers = await self.federation.discover_peers(node.host, node.port)
                    for peer in new_peers:
                        if peer.node_id != self.federation.local_node_id:
                            # Attempt handshake with newly discovered peer
                            await self.federation.handshake(peer.host, peer.port)
                            
            except Exception as e:
                logger.error(f"[FederationManager] Discovery loop error: {e}")
                
            await asyncio.sleep(self.check_interval * 2)

    async def _health_analytics_loop(self):
        """Monitor the 'Collective Resilience' of the mesh."""
        while self.is_running:
            try:
                stats = self.federation.get_federation_stats()
                online_count = stats.get("connected_nodes", 0)
                total_count = stats.get("total_nodes", 0)
                
                resilience_score = (online_count / total_count * 100) if total_count > 0 else 0
                
                logger.info(f"[FederationManager] Mesh Status: {online_count}/{total_count} nodes online. Resilience: {resilience_score:.1f}%")
                
                # If resilience is low and we have seeds, try to reconnect
                if resilience_score < 50 and total_count > 0:
                    logger.warning("[FederationManager] Low resilience detected. Attempting emergency reconnection...")
                    for node in self.federation.registry.connected_nodes.values():
                        if node.status != "online":
                            await self.federation.handshake(node.host, node.port)
                            
            except Exception as e:
                logger.error(f"[FederationManager] Health analytics error: {e}")
                
            await asyncio.sleep(self.check_interval)

    def get_collective_status(self) -> dict:
        """Return the health of the entire multi-swarm mesh."""
        if not self.federation:
            return {"status": "inactive"}
            
        stats = self.federation.get_federation_stats()
        online_nodes = self.federation.registry.get_online_nodes()
        
        return {
            "mesh_id": "collective_singularity_v1",
            "online_nodes": stats.get("connected_nodes", 0),
            "total_nodes": stats.get("total_nodes", 0),
            "resilience": (stats.get("connected_nodes", 0) / stats.get("total_nodes", 1) * 100) if stats.get("total_nodes", 0) > 0 else 0,
            "peers": [n.to_dict() for n in online_nodes]
        }

# Singleton instance
_manager: Optional[FederationManager] = None

def get_federation_manager() -> FederationManager:
    global _manager
    if _manager is None:
        _manager = FederationManager()
    return _manager
