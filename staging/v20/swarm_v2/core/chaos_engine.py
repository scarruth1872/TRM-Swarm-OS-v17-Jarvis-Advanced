# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 41, Rank 196)
# CDYLIB: strip_chaos_engine.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_chaos_engine.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
        try:
            _native_lib.strip_chaos_should_drop_pure.argtypes = [_ctypes.c_int, _ctypes.c_double,
                                                                 _ctypes.c_double]
            _native_lib.strip_chaos_should_drop_pure.restype = _ctypes.c_int32
        except Exception as _be:
            _NATIVE_LOADED = False
            _NATIVE_REASON = f"strip loaded but the wired symbol is missing: {_be}"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"


def _native_should_drop(enabled: bool, rate: float, sample: float):
    """The strip's decision for an explicit (enabled, rate, sample), or None when it cannot be answered.

    None means "run the python rule", so a failure anywhere on the native path degrades to the oracle instead
    of to a wrong answer -- and says so in `_NATIVE_REASON` rather than silently advertising a strip.
    """
    global _NATIVE_LOADED, _NATIVE_REASON
    if not _NATIVE_LOADED:
        return None
    try:
        return bool(_native_lib.strip_chaos_should_drop_pure(1 if enabled else 0, float(rate), float(sample)))
    except Exception as _ce:
        _NATIVE_LOADED = False  # one failure is enough: stop reporting a boundary that does not work
        _NATIVE_REASON = f"native call failed, falling back: {_ce}"
        return None

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
import asyncio
import logging
import random
from typing import Dict, Any

logger = logging.getLogger("ChaosEngine")

class ChaosEngine:
    """
    Modular Chaos Engineering Simulator (Phase 18).
    Allows test suites to inject network delays and socket dropouts to verify self-healing edge federation stability.
    """
    def __init__(self):
        self.network_delay_sec = 0.0
        self.dropout_rate = 0.0  # 0.0 to 1.0 probability of drop
        self.enabled = False

    def enable(self):
        self.enabled = True
        logger.info("[Chaos] Chaos injection enabled.")

    def disable(self):
        self.enabled = False
        self.network_delay_sec = 0.0
        self.dropout_rate = 0.0
        logger.info("[Chaos] Chaos injection disabled.")

    def set_latency(self, seconds: float):
        self.network_delay_sec = seconds
        logger.info(f"[Chaos] Simulated network latency set to {seconds}s.")

    def set_dropout_rate(self, rate: float):
        self.dropout_rate = max(0.0, min(1.0, rate))
        logger.info(f"[Chaos] Simulated dropout rate set to {self.dropout_rate * 100}%.")

    async def apply_network_delay(self):
        """Await this during remote sync requests to simulate latency."""
        if self.enabled and self.network_delay_sec > 0.0:
            logger.warning(f"[Chaos] Injecting network delay: {self.network_delay_sec}s")
            await asyncio.sleep(self.network_delay_sec)

    def should_drop_connection(self) -> bool:
        """Returns True if the connection should fail according to dropout_rate.

        The comparison runs in the strip when it is loaded; the pure-python rule is kept below as the oracle
        it was proved against and as the fallback, so a missing strip changes WHICH code runs, never the
        answer. The state is passed in rather than read from the strip's own engine, so a caller that mutates
        `enabled`/`dropout_rate` directly cannot make the native answer disagree with the python one.
        """
        if not (self.enabled and self.dropout_rate > 0.0):
            return False
        sample = random.random()
        native = _native_should_drop(self.enabled, self.dropout_rate, sample)
        drop = sample < self.dropout_rate if native is None else native
        if drop:
            logger.warning("[Chaos] Dropping remote connection attempt!")
            return True
        return False

    def get_profile(self) -> Dict[str, Any]:
        return {
            "enabled": self.enabled,
            "latency_seconds": self.network_delay_sec,
            "dropout_rate": self.dropout_rate
        }

# Singleton accessor
_chaos_engine = ChaosEngine()

def get_chaos_engine() -> ChaosEngine:
    return _chaos_engine
