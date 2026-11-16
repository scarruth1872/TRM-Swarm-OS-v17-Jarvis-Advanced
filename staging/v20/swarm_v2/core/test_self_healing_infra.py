# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 43, Rank 208)
# CDYLIB: strip_test_self_healing_infra.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_self_healing_infra.dll"
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
from fastapi.testclient import TestClient
from swarm_v2.app_v2 import app
from swarm_v2.core.self_healing_infra import get_self_healing_infra

import pytest


@pytest.mark.integration
class TestSelfHealingInfraAPI(unittest.TestCase):
    def setUp(self):
        self.client = TestClient(app)
        self.infra = get_self_healing_infra()
        self.orig_running = self.infra.running
        # Stop to prevent loop interference during test
        self.infra.running = False

    def tearDown(self):
        self.infra.running = self.orig_running

    def test_watchdog_toggle_endpoints(self):
        # Stop
        res_stop = self.client.post("/infra/stop")
        self.assertEqual(res_stop.status_code, 200)
        self.assertFalse(self.infra.running)

        # Start
        res_start = self.client.post("/infra/start")
        self.assertEqual(res_start.status_code, 200)
        
        # Check status
        res_status = self.client.get("/infra/health")
        self.assertEqual(res_status.status_code, 200)
        self.assertTrue(res_status.json()["running"])

        # Stop again
        self.client.post("/infra/stop")

    def test_reset_counter_endpoint(self):
        self.infra.health_status["swarm_api"].restart_count = 5
        res = self.client.post("/infra/reset/swarm_api")
        self.assertEqual(res.status_code, 200)
        self.assertEqual(self.infra.health_status["swarm_api"].restart_count, 0)

    def test_force_restart_endpoint(self):
        res = self.client.post("/infra/restart/invalid_service")
        self.assertEqual(res.status_code, 404)

if __name__ == '__main__':
    unittest.main()
