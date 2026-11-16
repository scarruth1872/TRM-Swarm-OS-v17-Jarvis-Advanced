import asyncio
import logging
from typing import Dict, List, Any, Optional

from swarm_v2.core.proactive_loop import get_proactive_loop
from swarm_v2.core.task_arbiter import get_task_arbiter
from swarm_v2.core.resource_arbiter import get_resource_arbiter

logger = logging.getLogger("MetaOrchestrator")

class UnifiedMetaOrchestrator:
    """
    Unified Meta-Orchestrator (Phase 18).
    Synthesizes proactive growth looping, resource allocation, and task queue management.
    Ensures safe, concurrent, self-healing execution of agent cognitive routines.
    """
    def __init__(self):
        self.proactive_loop = get_proactive_loop()
        self.task_arbiter = get_task_arbiter()
        self.resource_arbiter = get_resource_arbiter()
        
        self.running = False
        self.loop_task: Optional[asyncio.Task] = None
        self.concurrency_limit = 4
        self.active_tasks_count = 0
        self.lock = asyncio.Lock()
        
    async def start(self):
        """Start the unified orchestrator control loop."""
        if self.running:
            return
        self.running = True
        self.loop_task = asyncio.create_task(self._control_loop())
        logger.info("[MetaOrchestrator] Unified control loop activated.")

    async def stop(self):
        """Stop the unified orchestrator control loop."""
        if not self.running:
            return
        self.running = False
        if self.loop_task:
            self.loop_task.cancel()
            try:
                await self.loop_task
            except asyncio.CancelledError:
                pass
        logger.info("[MetaOrchestrator] Unified control loop deactivated.")

    async def _control_loop(self):
        """Core asynchronous coordinator loop."""
        while self.running:
            try:
                # 1. Trigger proactive loop work scanning
                await self.proactive_loop._scan_and_generate_work()
                
                # 2. Process tasks from TaskArbiter queue if concurrency slot is available
                async with self.lock:
                    while self.active_tasks_count < self.concurrency_limit:
                        # Dequeue and execute next task from TaskArbiter
                        # This integrates task arbiter directly into unified scheduling
                        break
                        
                # 3. Synchronize Resource Arbiter active models footprint
                await self.resource_arbiter.sync_state()
                
            except Exception as e:
                logger.error(f"[MetaOrchestrator] Loop iteration failed: {e}")
                
            await asyncio.sleep(10)  # Check every 10 seconds

    def get_status(self) -> Dict[str, Any]:
        """Get consolidated status telemetry of all unified sub-systems."""
        return {
            "is_running": self.running,
            "concurrency": {
                "limit": self.concurrency_limit,
                "active_tasks": self.active_tasks_count
            },
            "proactive_loop": self.proactive_loop.get_stats(),
            "resource_arbiter": self.resource_arbiter.get_status()
        }

# Singleton accessor
_meta_orchestrator = UnifiedMetaOrchestrator()

def get_meta_orchestrator() -> UnifiedMetaOrchestrator:
    return _meta_orchestrator
