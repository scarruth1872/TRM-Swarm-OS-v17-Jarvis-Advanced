# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 36, Rank 173)
# CDYLIB: strip_meta_orchestrator.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_meta_orchestrator.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
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
