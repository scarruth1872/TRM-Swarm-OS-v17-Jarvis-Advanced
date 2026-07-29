import asyncio
import json
import os
import aiohttp
import logging
from datetime import datetime
from typing import Dict, List, Any, Optional

import hashlib
import hmac
from swarm_v2.core.global_memory import get_global_memory
from swarm_v2.core.telemetry import get_telemetry

logger = logging.getLogger("MemorySynchronizer")

class MemorySynchronizer:
    """
    Phase 17: Collective Singularity Synchronizer.
    
    Responsible for synchronizing genetic proposals and learned patterns 
    across federated Swarm OS instances. This creates a global intelligence 
    fabric where one swarm's optimization benefits all nodes in the federation.
    """
    
    def __init__(self, sync_interval: int = 3600):
        self.sync_interval = sync_interval
        self.federation_nodes: List[str] = []
        self.is_running = False
        self.memory = get_global_memory()
        self.telemetry = get_telemetry()
        self.secret = os.getenv("SWARM_FEDERATION_SECRET", "default_secret_change_me")
        
        # Load bootstrap nodes from environment if available
        bootstrap = os.environ.get("SWARM_FEDERATION_BOOTSTRAP", "")
        if bootstrap:
            self.federation_nodes = [node.strip() for node in bootstrap.split(",")]

    def _generate_hmac(self, data: dict) -> str:
        """Generate HMAC signature for a data dictionary."""
        message = json.dumps(data, sort_keys=True).encode()
        return hmac.new(self.secret.encode(), message, hashlib.sha256).hexdigest()

    async def start(self):
        """Start the synchronization background loop."""
        if self.is_running:
            return
        
        self.is_running = True
        logger.info(f"MemorySynchronizer started with {len(self.federation_nodes)} bootstrap nodes.")
        
        while self.is_running:
            try:
                await self.synchronize_all()
            except Exception as e:
                logger.error(f"Sync cycle failed: {e}")
            
            await asyncio.sleep(self.sync_interval)

    async def stop(self):
        """Gracefully stop the synchronizer."""
        self.is_running = False
        logger.info("MemorySynchronizer stopping...")

    async def synchronize_all(self):
        """Perform a full synchronization pass with all federated nodes."""
        if not self.federation_nodes:
            logger.debug("No federation nodes configured. Skipping sync.")
            return

        logger.info(f"Synchronizing with {len(self.federation_nodes)} nodes...")
        
        # 1. Fetch local genetic proposals
        local_proposals = await self._get_local_proposals()
        
        # 2. Push/Pull with each node
        for node_url in self.federation_nodes:
            await self._sync_with_node(node_url, local_proposals)

    async def _get_local_proposals(self) -> List[Dict[str, Any]]:
        """Retrieve verified genetic proposals from the local mutation engine."""
        from swarm_v2.core.mutation_engine import get_mutation_engine
        engine = get_mutation_engine()
        return engine.get_proposals()

    async def _sync_with_node(self, node_url: str, local_data: List[Dict[str, Any]]):
        """Exchange memory shards with a specific federated node."""
        try:
            async with aiohttp.ClientSession() as session:
                # Exchange genetic blueprints (Signed request)
                sync_data = {"timestamp": datetime.now().isoformat(), "swarm_id": "local"}
                signature = self._generate_hmac(sync_data)
                
                endpoint = f"{node_url.rstrip('/')}/evolution/proposals"
                async with session.get(endpoint, params={"data": json.dumps(sync_data), "signature": signature}, timeout=10) as resp:
                    if resp.status == 200:
                        remote_proposals = await resp.json()
                        await self._integrate_remote_proposals(remote_proposals)
                        
                # Exchange telemetry and health stats (Signed request)
                telemetry_endpoint = f"{node_url.rstrip('/')}/swarm/telemetry"
                async with session.get(telemetry_endpoint, params={"data": json.dumps(sync_data), "signature": signature}, timeout=5) as resp:
                    if resp.status == 200:
                        remote_telemetry = await resp.json()
                        await self.telemetry.record_external_metric(node_url, remote_telemetry)

        except Exception as e:
            logger.warning(f"Failed to sync with node {node_url}: {e}")

    async def _integrate_remote_proposals(self, proposals: List[Dict[str, Any]]):
        """Evaluate and potentially ingest genetic proposals from remote swarms."""
        for proposal in proposals:
            if proposal.get("status") == "verified":
                logger.info(f"Ingesting remote verified proposal: {proposal.get('id')}")
                # Integration logic will be hardened in Phase 17
                self.memory.contribute(
                    content=json.dumps(proposal),
                    author="federation",
                    author_role="federated_swarm",
                    memory_type="remote_proposal",
                    tags=["federated", "genetic_proposal"]
                )

_instance: Optional[MemorySynchronizer] = None

def get_memory_synchronizer() -> MemorySynchronizer:
    global _instance
    if _instance is None:
        _instance = MemorySynchronizer()
    return _instance
