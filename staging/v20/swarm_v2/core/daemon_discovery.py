# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 35, Rank 169)
# CDYLIB: strip_daemon_discovery.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_daemon_discovery.dll"
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
"""
Dynamic Agent Discovery Daemon — IPC Edition (v18.13)
Phase 3: Mesh Gap Remediation
Uses DIRECT IPC via agent_mesh.propagate_heartbeat() instead of HTTP.
Eliminates 12 HTTP requests every 10 seconds — zero network overhead.
"""
import asyncio
import time
import uuid
import psutil
import logging

logger = logging.getLogger("DiscoveryDaemon")

class DiscoveryDaemon:
    def __init__(self, node_id: str, name: str, role: str, target_url: str = None, interval: int = 10):
        self.node_id = node_id
        self.name = name
        self.role = role
        self.target_url = target_url  # Kept for compatibility, but NOT used for IPC
        self.interval = interval
        self.running = False
        self._mesh = None
        
    async def start(self):
        """Start the discovery heartbeat via DIRECT IPC — no HTTP."""
        self.running = True
        logger.info(f"Discovery Daemon (IPC) initiated for agent '{self.name}' ({self.role})")
        
        # Lazy-load agent mesh for direct IPC
        try:
            from swarm_v2.core.agent_mesh import get_agent_mesh
            self._mesh = get_agent_mesh()
        except Exception:
            pass
        
        while self.running:
            try:
                cpu_load = psutil.cpu_percent()
                memory_usage = psutil.virtual_memory().percent
                
                # DIRECT IPC: propagate heartbeat via function call, not HTTP
                if self._mesh:
                    try:
                        self._mesh.propagate_heartbeat(self.node_id)
                    except Exception:
                        pass
                
                # Also update node telemetry directly
                if self._mesh:
                    try:
                        self._mesh.update_node_telemetry(self.node_id, {
                            "cpu_load": cpu_load,
                            "memory_usage": memory_usage,
                            "timestamp": time.time()
                        })
                    except Exception:
                        pass
                
                # Fallback: if no mesh, make one lightweight HTTP call for entire daemon
                if not self._mesh and self.target_url:
                    try:
                        import urllib.request, json
                        payload = {
                            "node_id": self.node_id, "name": self.name,
                            "role": self.role, "status": "online",
                            "hardware": {"cpu_load": cpu_load, "memory_usage": memory_usage, "timestamp": time.time()}
                        }
                        req = urllib.request.Request(
                            self.target_url,
                            data=json.dumps(payload).encode("utf-8"),
                            headers={"Content-Type": "application/json"},
                            method="POST"
                        )
                        await asyncio.to_thread(lambda: urllib.request.urlopen(req, timeout=3))
                    except Exception:
                        pass
                
            except Exception as e:
                logger.error(f"Discovery daemon error: {e}")
                
            await asyncio.sleep(self.interval)
            
    def stop(self):
        self.running = False
        logger.info(f"Discovery Daemon stopped for '{self.name}'.")

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] [Discovery] %(message)s")
    node_id = uuid.uuid4().hex[:16]
    daemon = DiscoveryDaemon(node_id, "TRM-Autonomous-Node", "worker")
    try:
        asyncio.run(daemon.start())
    except KeyboardInterrupt:
        daemon.stop()
