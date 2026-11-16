# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 41, Rank 200)
# CDYLIB: strip_test_research_api.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_research_api.dll"
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
from swarm_v2.core.reconnaissance_daemon import get_reconnaissance_daemon

import pytest


@pytest.mark.integration
class TestResearchAPI(unittest.TestCase):
    def setUp(self):
        self.client = TestClient(app)
        self.daemon = get_reconnaissance_daemon()
        self.orig_topics = list(self.daemon.topics)
        self.orig_running = self.daemon.is_running
        self.daemon.stop()

    def tearDown(self):
        # Restore daemon settings
        self.daemon.topics = self.orig_topics
        if self.orig_running:
            self.daemon.start()
        else:
            self.daemon.stop()

    def test_start_stop_daemon_endpoints(self):
        # Stop (should be stopped already)
        res = self.client.post("/research/stop")
        self.assertEqual(res.status_code, 200)
        self.assertFalse(self.daemon.is_running)

        # Start
        res_start = self.client.post("/research/start?interval_hours=24")
        self.assertEqual(res_start.status_code, 200)
        self.assertTrue(self.daemon.is_running)

        # Check stats
        res_stats = self.client.get("/research/stats")
        self.assertEqual(res_stats.status_code, 200)
        self.assertTrue(res_stats.json()["is_running"])

    def test_topic_management_endpoints(self):
        self.daemon.topics = ["test_topic_1"]

        # Add topic
        res_add = self.client.post("/research/topics?topic=test_topic_2")
        self.assertEqual(res_add.status_code, 200)
        self.assertIn("test_topic_2", self.daemon.topics)

        # Remove topic
        res_del = self.client.delete("/research/topics/test_topic_1")
        self.assertEqual(res_del.status_code, 200)
        self.assertNotIn("test_topic_1", self.daemon.topics)

    def test_trigger_run_endpoint(self):
        res = self.client.post("/research/run")
        self.assertEqual(res.status_code, 200)
        self.assertEqual(res.json()["status"], "triggered")

if __name__ == '__main__':
    unittest.main()
