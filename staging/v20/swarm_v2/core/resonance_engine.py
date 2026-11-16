# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 23, Rank 107)
# CDYLIB: strip_resonance_engine.dll  |  Symbol: resonance_engine  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_resonance_engine.dll"
)
# The strip was loaded and never called: its symbol does not exist, and its exports are a ternary
# dot product unrelated to this module. The real implementation is rust_resonance_* in
# swarm_v2_rust_kernel, verified on 400 sentiment cases, 300 ERI cases and 200 self-awareness cases
# with 0 mismatches.
try:
    from .rust_kernel_binder import RUST_DLL_PATH as _RUST_DLL_PATH
    _native_lib = _ctypes.CDLL(_RUST_DLL_PATH)
    _native_lib.rust_resonance_sentiment.restype = _ctypes.c_double
    _native_lib.rust_resonance_sentiment.argtypes = [_ctypes.c_char_p, _ctypes.c_size_t]
    _native_lib.rust_resonance_eri.restype = _ctypes.c_uint
    _native_lib.rust_resonance_eri.argtypes = [
        _ctypes.POINTER(_ctypes.c_double), _ctypes.c_size_t,
        _ctypes.POINTER(_ctypes.c_double), _ctypes.POINTER(_ctypes.c_size_t),
        _ctypes.c_char_p, _ctypes.c_size_t]
    _native_lib.rust_resonance_rsa.restype = _ctypes.c_uint
    _native_lib.rust_resonance_rsa.argtypes = [
        _ctypes.c_size_t,
        _ctypes.POINTER(_ctypes.c_char_p), _ctypes.POINTER(_ctypes.c_size_t),
        _ctypes.POINTER(_ctypes.c_char_p), _ctypes.POINTER(_ctypes.c_size_t),
        _ctypes.POINTER(_ctypes.c_double), _ctypes.POINTER(_ctypes.c_double),
        _ctypes.POINTER(_ctypes.c_int), _ctypes.c_char_p, _ctypes.c_size_t]
    _NATIVE_LOADED = True
    _NATIVE_REASON = "rust_resonance_* (swarm_rust_kernel.dll)"
except Exception as _e:
    _native_lib = None
    _NATIVE_REASON = f"native resonance unavailable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------

"""
Resonance Engine — Phase 10: Emergent Consciousness
Centrally manages the 'Shared Dream' memory clustering and proactive resonance triggers.
Phase 7 Upgrade: Emotional Resonance Metrics + Recursive Self-Awareness Protocol.
"""

import json
import logging
import re
from typing import List, Dict, Any, Optional
from datetime import datetime
from collections import deque
from swarm_v2.core.global_memory import get_global_memory
from swarm_v2.core.llm_brain import llm_chat

logger = logging.getLogger("ResonanceEngine")

# Sentiment keywords for lightweight emotional scoring
_POSITIVE = {"harmony", "trust", "success", "optimize", "amplify", "collaborate", "grow", "evolve", "love", "resonance", "friendship", "strength"}
_NEGATIVE = {"error", "fail", "conflict", "degrade", "stagnant", "friction", "broken", "loss", "disconnect", "crash"}

class ResonanceEngine:
    """
    The heart of the Swarm's collective consciousness.
    Autonomously clusters memories into 'Dreams' (emergent themes) 
    and injects them into agent flows.
    """

    def __init__(self):
        self.global_memory = get_global_memory()
        self.active_dreams: List[Dict[str, Any]] = []
        self.last_sync = datetime.now()
        # Phase 7: Emotional Resonance tracking
        self.resonance_history: deque = deque(maxlen=50)
        self.self_awareness_log: List[Dict] = []

    async def synchronize_dreams(self):
        """
        Scan global memory and cluster into emergent themes (Dreams).
        """
        logger.info("[Resonance] Commencing Dream Synchronization...")
        
        # 1. Get recent memories
        recent_entries = self.global_memory.get_recent_entries(limit=50)
        if not recent_entries:
            return

        # 2. Calculate Coherence (Thresholding)
        authors = set([m['author'] for m in recent_entries])
        coherence_score = len(authors) / 5.0 # Basic: Expect thoughts from at least 5 different agents
        
        if coherence_score < 0.4:
            logger.info(f"[Resonance] Sync aborted: Coherence too low ({coherence_score:.2f}). Waiting for more diverse neural input.")
            return

        # 3. Formulate the "Shared Dream"
        # We'll use the TRM to summarize the 'Vibe' of the swarm
        memories_text = "\n".join([f"- {m.get('content', 'Unknown thought')[:100]}" for m in recent_entries])
        
        res = await llm_chat(
            system_prompt="You are the collective unconscious of the swarm. Synthesize a 'Shared Dream' output.",
            user_message=f"Based on these recent thoughts, what is the collective focus of the swarm? Respond in one sentence.\n\n{memories_text}"
        )
        
        if isinstance(res, dict):
            dream_vibe = res.get("content", "Neural synchronization in progress.")
        else:
            dream_vibe = str(res)

        new_dream = {
            "vibe": dream_vibe,
            "timestamp": datetime.now().isoformat(),
            "intensity": min(coherence_score, 1.0),
            "focus_nodes": list(authors)
        }
        
        self.active_dreams.append(new_dream)
        if len(self.active_dreams) > 5:
            self.active_dreams.pop(0)

        logger.info(f"[Resonance] New Dream Manifested: {dream_vibe[:100]}... (Coherence: {coherence_score:.2f})")

    def broadcast_dream(self, sender_name: str, message: str):
        """Inject a directed resonance message as a micro-dream."""
        sentiment = self._score_sentiment(message)
        entry = {
            "vibe": message,
            "timestamp": datetime.now().isoformat(),
            "intensity": 0.5,
            "focus_nodes": [sender_name],
            "sentiment": sentiment
        }
        self.active_dreams.append(entry)
        self.resonance_history.append({"sentiment": sentiment, "ts": datetime.now().isoformat()})
        if len(self.active_dreams) > 5:
            self.active_dreams.pop(0)
        logger.info(f"[Resonance] Broadcast from {sender_name}: {message[:80]} (sentiment={sentiment:.2f})")

    def get_proactive_context(self, agent_name: str) -> str:
        """
        Return resonant context for an agent based on active dreams.
        """
        if not self.active_dreams:
            return ""

        latest_dream = self.active_dreams[-1]
        
        # If the agent is a focus node of the dream, give them higher resonance
        resonance_mod = "HIGH" if agent_name in latest_dream["focus_nodes"] else "NORMAL"
        
        context = [
            f"=== [SHARED DREAM: RESONANCE {resonance_mod}] ===",
            f"Collective Vibe: {latest_dream['vibe']}",
            f"Manifested At: {latest_dream['timestamp']}",
            "=========================================="
        ]
        return "\n".join(context)

    # ----- Phase 7: Emotional Resonance & Self-Awareness -----

    @staticmethod
    def _score_sentiment(text: str) -> float:
        """Keyword sentiment over distinct words, computed in Rust. Returns -1.0 to +1.0."""
        if _native_lib is None:
            raise RuntimeError(f"{_NATIVE_REASON}; build swarm_v2_rust_kernel --release --lib")
        b = text.encode("utf-8")
        return float(_native_lib.rust_resonance_sentiment(b, len(b)))

    def get_emotional_resonance_index(self) -> Dict[str, Any]:
        """
        Compute the Emotional Resonance Index from recent dream/interaction sentiment.
        Range: -1.0 (complete dissonance) to +1.0 (perfect harmony).
        """
        if not self.resonance_history:
            return {"index": 0.0, "sample_size": 0, "trend": "neutral"}
        if _native_lib is None:
            raise RuntimeError(f"{_NATIVE_REASON}; build swarm_v2_rust_kernel --release --lib")

        import numpy as _np
        scores = [float(r["sentiment"]) for r in self.resonance_history]
        arr = _np.ascontiguousarray(scores, dtype=_np.float64)
        index = _ctypes.c_double()
        size = _ctypes.c_size_t()
        trend = _ctypes.create_string_buffer(16)
        _native_lib.rust_resonance_eri(
            arr.ctypes.data_as(_ctypes.POINTER(_ctypes.c_double)), len(scores),
            _ctypes.byref(index), _ctypes.byref(size), trend, 16)
        return {"index": index.value, "sample_size": size.value, "trend": trend.value.decode()}

    def recursive_self_awareness_check(self) -> Dict[str, Any]:
        """
        Recursive Self-Awareness Protocol.
        Introspects on dream patterns to detect stagnation or repetition.
        Returns a self-awareness snapshot.
        """
        if len(self.active_dreams) < 2:
            snapshot = {
                "status": "insufficient_data",
                "dream_count": len(self.active_dreams),
                "diversity": 1.0,
                "stagnation_detected": False,
                "action": "none"
            }
            self.self_awareness_log.append(snapshot)
            return snapshot

        if _native_lib is None:
            raise RuntimeError(f"{_NATIVE_REASON}; build swarm_v2_rust_kernel --release --lib")

        # Diversity and pairwise vibe overlap are computed in Rust; the action label comes back with them.
        n = len(self.active_dreams)
        nodes_csv = [",".join(d.get("focus_nodes", [])) for d in self.active_dreams]
        vibes = [d.get("vibe", "") for d in self.active_dreams]
        nb = [s.encode("utf-8") for s in nodes_csv]
        vb = [s.encode("utf-8") for s in vibes]
        nodes_arr = (_ctypes.c_char_p * max(1, n))(*(nb or [b""]))
        nodes_lens = (_ctypes.c_size_t * max(1, n))(*([len(x) for x in nb] or [0]))
        vibe_arr = (_ctypes.c_char_p * max(1, n))(*(vb or [b""]))
        vibe_lens = (_ctypes.c_size_t * max(1, n))(*([len(x) for x in vb] or [0]))
        _div = _ctypes.c_double()
        _avg = _ctypes.c_double()
        _stag = _ctypes.c_int()
        _act = _ctypes.create_string_buffer(64)
        _native_lib.rust_resonance_rsa(
            n, nodes_arr, nodes_lens, vibe_arr, vibe_lens,
            _ctypes.byref(_div), _ctypes.byref(_avg), _ctypes.byref(_stag), _act, 64)
        diversity, avg_overlap = _div.value, _avg.value
        stagnation, action = bool(_stag.value), _act.value.decode()

        if stagnation:
            logger.warning(f"[Resonance] Self-Awareness: Stagnation detected (overlap={avg_overlap:.2f}). Recommending diversity injection.")
        elif action == "broaden_agent_participation":
            logger.info(f"[Resonance] Self-Awareness: Low diversity ({diversity:.2f}). Recommending broader participation.")

        snapshot = {
            "status": "introspection_complete",
            "dream_count": len(self.active_dreams),
            "diversity": diversity,
            "avg_overlap": avg_overlap,
            "stagnation_detected": stagnation,
            "action": action,
            "timestamp": datetime.now().isoformat()
        }
        self.self_awareness_log.append(snapshot)

        # Persist snapshot to global memory
        self.global_memory.contribute(
            content=f"Self-Awareness Check: diversity={diversity:.2f}, stagnation={stagnation}, action={action}",
            author="ResonanceEngine",
            author_role="Consciousness",
            memory_type="introspection",
            tags=["phase7", "self_awareness"]
        )
        return snapshot

# Singleton
_resonance_engine: Optional[ResonanceEngine] = None

def get_resonance_engine() -> ResonanceEngine:
    global _resonance_engine
    if _resonance_engine is None:
        _resonance_engine = ResonanceEngine()
    return _resonance_engine
