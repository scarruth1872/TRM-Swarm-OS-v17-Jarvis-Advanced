# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 41, Rank 198)
# CDYLIB: strip_llm_orchestrator.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_llm_orchestrator.dll"
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
LLM Orchestrator - Phase 2
Handles strategic routing of cognitive tasks based on complexity,
priority, and required reasoning depth.
"""

from typing import Dict, Any, List, Optional
from swarm_v2.core.llm_brain import get_active_model

class LLMOrchestrator:
    def __init__(self):
        self.executive_model = "gemma4:e2b"
        self.reasoning_model = "deepseek-r1:8b"
        self.thresholds = {
            "complexity_min": 0.7,
            "security_critical": True
        }

    def determine_routing(self, task: str, complexity_score: float = 0.5) -> str:
        """
        Logic to decide which model should handle the task.
        Returns the model name.
        """
        # 1. Check for explicit overrides in the task text
        if "[DEEP_REASONING]" in task.upper():
            return self.reasoning_model
        
        # 2. Check complexity score (calculated by Pulse or Logic agents)
        if complexity_score >= self.thresholds["complexity_min"]:
            return self.reasoning_model
        
        # 3. Keyword-based heuristic
        high_reasoning_keywords = ["architecture", "refactor", "security audit", "optimize", "proof"]
        if any(kw in task.lower() for kw in high_reasoning_keywords):
            return self.reasoning_model

        # Default to the agile executive model
        return self.executive_model

    def synthesize_context(self, agent_responses: List[Dict[str, Any]]) -> str:
        """
        Aggregates multiple agent perspectives into a single 'Unified Truth' 
        for the final executive completion.
        """
        unified = "# Unified Swarm Intelligence Context\n\n"
        for resp in agent_responses:
            agent = resp.get("agent", "Unknown")
            content = resp.get("content", "")
            unified += f"### Perspective: {agent}\n{content}\n\n"
        
        return unified

# Singleton
_orchestrator: Optional[LLMOrchestrator] = None

def get_orchestrator() -> LLMOrchestrator:
    global _orchestrator
    if _orchestrator is None:
        _orchestrator = LLMOrchestrator()
    return _orchestrator
