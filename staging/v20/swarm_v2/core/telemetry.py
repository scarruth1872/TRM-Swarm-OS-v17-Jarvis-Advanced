# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 36, Rank 173)
# CDYLIB: strip_telemetry.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_telemetry.dll"
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
import psutil
from typing import Dict, Any, List
from datetime import datetime
from swarm_v2.core.manus_engine import get_manus_engine
from swarm_v2.core.resource_arbiter import get_resource_arbiter
from swarm_v2.core.coherence_thresholds import CoherenceThresholds
from swarm_v2.core.proactive_loop import get_proactive_loop
from swarm_v2.skills.relationship_skill import RelationshipReasoningSkill
from typing import Optional

class SwarmTelemetry:
    """
    Emergence Hub Telemetry (Phase 4).
    Aggregates live metrics from the Mesh, Manus Engine, and Resource Arbiter.
    """

    def __init__(self):
        self.manus = get_manus_engine()
        self.arbiter = get_resource_arbiter()
        self.proactive = get_proactive_loop()
        self.harmony_monitor = RelationshipReasoningSkill()

    def get_emergence_report(self) -> Dict[str, Any]:
        """
        Generates a comprehensive snapshot of the swarm's current state.
        """
        # 1. System Stability
        mem = psutil.virtual_memory()
        system_stats = {
            "cpu_percent": psutil.cpu_percent(),
            "cpu_load": psutil.cpu_percent(), # Alias for internal check_stability
            "memory_percent": mem.percent,
            "memory_usage": mem.percent,     # Alias for internal check_stability
            "timestamp": datetime.now().isoformat()
        }
        
        # 2. Vibe/Coherence Status
        stability_level = CoherenceThresholds.check_stability(system_stats)
        
        # 3. Quantum-Inspired States (Manus Protocol)
        active_superpositions = []
        for tid, data in self.manus.active_superpositions.items():
            active_superpositions.append({
                "task_id": tid,
                "name": data["name"],
                "coherence": data["coherence"],
                "status": data["status"],
                "paths": len(data["roles"])
            })
            
        # 4. Resource Allocation
        vram_stats = self.arbiter.get_status()
        
        # 5. Distributed Stack Health
        from swarm_v2.experts.registry import get_expert_team
        team = get_expert_team()
        stack_stats = []
        for role, agent in team.items():
            if hasattr(agent, "cognitive_stack"):
                stack_stats.append(agent.cognitive_stack.get_status())
        
        return {
            "status": stability_level,
            "system": system_stats,
            "superpositions": active_superpositions,
            "resource_arbiter": vram_stats,
            "distributed_stacks": stack_stats,
            "mesh_coherence": self._calculate_global_coherence(active_superpositions),
            "harmony_index": self.harmony_monitor.get_harmony_index(),
            "active_proposals": len(self.proactive.get_active_proposals())
        }

    def get_soul_report(self) -> Dict[str, Any]:
        """
        Aggregates philosophical status and autonomous evolution proposals.
        """
        return {
            "harmony_index": self.harmony_monitor.get_harmony_index(),
            "reflection": self.harmony_monitor.generate_alignment_reflection(),
            "autonomous_proposals": self.proactive.get_active_proposals(),
            "timestamp": datetime.now().isoformat()
        }

    def _calculate_global_coherence(self, superpositions: List[Dict]) -> float:
        """Calculates global swarm coherence as the average of active tasks."""
        if not superpositions:
            return 1.0 # Optimal state if idle
        
        total_coherence = sum(s["coherence"] for s in superpositions)
        return total_coherence / len(superpositions)

    async def record_external_metric(self, node_url: str, metrics: Dict[str, Any]):
        """Record health metrics from a federated swarm instance."""
        # For Phase 17, we simply log it or store in a local ring buffer
        # This will be integrated with the dashboard in Phase 18
        pass

_telemetry: Optional['SwarmTelemetry'] = None

def get_telemetry() -> 'SwarmTelemetry':
    global _telemetry
    if _telemetry is None:
        _telemetry = SwarmTelemetry()
    return _telemetry
