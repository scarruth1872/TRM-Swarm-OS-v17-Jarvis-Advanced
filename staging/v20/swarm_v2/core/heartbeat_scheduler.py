# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 38, Rank 183)
# CDYLIB: strip_heartbeat_scheduler.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_heartbeat_scheduler.dll"
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
import os
import asyncio
import re
from datetime import datetime
from typing import Dict, Any, List, Optional
import logging

logger = logging.getLogger(__name__)

class HeartbeatScheduler:
    """
    Poller for HEARTBEAT.md to trigger proactive agent actions.
    """
    def __init__(self, heartbeat_file: str, engine_team: Dict[str, Any], interval: int = 60):
        self.heartbeat_file = heartbeat_file
        self.engine_team = engine_team
        self.interval = interval
        self.is_running = False

    async def start(self):
        self.is_running = True
        logger.info(f"HeartbeatScheduler started (Interval: {self.interval}s)")
        while self.is_running:
            try:
                await self.pulse()
            except Exception as e:
                logger.error(f"Heartbeat pulse failed: {e}")
            await asyncio.sleep(self.interval)

    def stop(self):
        self.is_running = False

    async def pulse(self):
        """Perform one pulse cycle: read, dispatch, update."""
        if not os.path.exists(self.heartbeat_file):
            return

        with open(self.heartbeat_file, "r") as f:
            content = f.read()

        # Find uncompleted tasks: - [ ] Task
        uncompleted = re.findall(r'-\s\[\s\]\s*(.*)', content)
        
        if not uncompleted:
            return

        logger.info(f"Heartbeat detected {len(uncompleted)} active goals/tasks.")
        
        # Dispatch the first uncompleted task to the Architect for coordination
        task = uncompleted[0].split('\n')[0].strip()
        architect = self.engine_team.get("Architect")
        
        if architect:
            logger.info(f"Dispatching proactive task: {task}")
            # We use 'self' sender to indicate internal proactive motivation
            await architect.process_task(f"PROACTIVE_HEARTBEAT: {task}", sender="self")
            
            # Mark as completed in file (primitive update)
            new_content = content.replace(f"- [ ] {task}", f"- [x] {task}")
            
            # Update last heartbeat
            ts = datetime.now().isoformat()
            new_content = re.sub(r'Last Heartbeat: .*', f'Last Heartbeat: {ts}', new_content)
            
            with open(self.heartbeat_file, "w") as f:
                f.write(new_content)

_scheduler: Optional[HeartbeatScheduler] = None

def get_heartbeat_scheduler(heartbeat_file: str = None, engine_team: Dict[str, Any] = None) -> HeartbeatScheduler:
    global _scheduler
    if _scheduler is None:
        if not heartbeat_file:
            heartbeat_file = os.path.join(os.getcwd(), "HEARTBEAT.md")
        _scheduler = HeartbeatScheduler(heartbeat_file, engine_team)
    return _scheduler
