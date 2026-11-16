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
