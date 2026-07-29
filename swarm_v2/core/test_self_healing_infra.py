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
