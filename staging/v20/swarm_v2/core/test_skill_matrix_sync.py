# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 42, Rank 204)
# CDYLIB: strip_test_skill_matrix_sync.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_skill_matrix_sync.dll"
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
import math
from unittest.mock import MagicMock
from swarm_v2.core.skill_matrix_sync import get_skill_matrix_sync

class TestSkillMatrixSync(unittest.TestCase):
    def setUp(self):
        self.sync = get_skill_matrix_sync()
        self.sync.proficiency_matrix.clear()
        self.sync._usage_tracker.clear()
        self.sync._skill_events.clear()

        self.mock_mem = MagicMock()
        self.mock_cqs = MagicMock()
        self.mock_broadcaster = MagicMock()
        self.sync.bind(self.mock_mem, self.mock_cqs, self.mock_broadcaster)

    def test_record_skill_usage(self):
        # Initial usage (success)
        self.sync.record_skill_usage("Devo", "Learned_Search", True)
        
        prof = self.sync.proficiency_matrix["Devo"]["Learned_Search"]
        # uses=1, successes=1 -> success_rate=1.0, math.log2(2) = 1.0 -> prof = 1.0
        self.assertEqual(prof, 1.0)
        
        # Second usage (failure)
        self.sync.record_skill_usage("Devo", "Learned_Search", False)
        prof_2 = self.sync.proficiency_matrix["Devo"]["Learned_Search"]
        # uses=2, successes=1 -> success_rate=0.5, math.log2(3) = 1.5849 -> prof = 0.5 * 1.5849 = 0.792
        self.assertAlmostEqual(prof_2, 0.792, places=3)

    def test_on_skill_created(self):
        skill = MagicMock()
        skill.skill_name = "TestSkill"
        skill.description = "Test Description"
        skill.instructions = "Instructions"
        skill.source = "file://test.md"
        skill.endpoints = {"/test": "GET"}

        self.sync.on_skill_created(skill)
        
        self.mock_mem.contribute.assert_called_once()
        self.mock_cqs.ingest_triples.assert_called()
        self.mock_broadcaster.push_event.assert_called_once()

if __name__ == '__main__':
    unittest.main()
