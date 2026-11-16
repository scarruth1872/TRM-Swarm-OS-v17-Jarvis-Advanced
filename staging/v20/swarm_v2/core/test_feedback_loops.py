# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 45, Rank 216)
# CDYLIB: strip_test_feedback_loops.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_feedback_loops.dll"
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
import unittest
from swarm_v2.core.feedback_loops import get_feedback_engine

class TestPhysicalFeedbackEngine(unittest.TestCase):
    def setUp(self):
        self.engine = get_feedback_engine()

    def test_focus_rules(self):
        # Focus mode
        status = self.engine.update_feed(0.85, True, 120.0)
        self.assertEqual(status["lighting_level"], 35)
        self.assertEqual(status["audio_volume"], 30)
        self.assertEqual(status["audio_mode"], "Lofi Deep Focus")

        # Relaxed mode
        status = self.engine.update_feed(0.3, True, 80.0)
        self.assertEqual(status["lighting_level"], 100)
        self.assertEqual(status["audio_volume"], 60)
        self.assertEqual(status["audio_mode"], "Ambient Relaxed")

        # Vacant / Absence mode
        status = self.engine.update_feed(0.5, False, 10.0)
        self.assertEqual(status["lighting_level"], 10)
        self.assertEqual(status["audio_volume"], 0)
        self.assertEqual(status["audio_mode"], "Muted / Studio Vacant")
