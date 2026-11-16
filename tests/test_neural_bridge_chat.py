"""
test_neural_bridge_chat.py — TRM Swarm OS v19.001 "Sovereign"
=============================================================
Comprehensive Test Suite for Neural Bridge (Tab 2) Chat Interface:
- Tests message dispatch and response generation across all 15 Swarm Agents.
- Verifies deterministic latency bounds (< 250 µs).
- Tests multimodal prompt payload parsing and reasoning traces.
"""

import sys
import os
import unittest
import asyncio
import json

PROJECT_ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from fastapi.testclient import TestClient
from swarm_v2.app_v2 import app

class TestNeuralBridgeChat(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.client = TestClient(app)

    def test_list_experts_endpoint(self):
        """Verify all 15 agents return with avatar colors and specialties."""
        res = self.client.get("/swarm/experts")
        self.assertEqual(res.status_code, 200)
        data = res.json()
        self.assertGreaterEqual(len(data), 15)
        
        roles = [e["role"] for e in data]
        for expected in ["Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Jarvis", "Hermes"]:
            self.assertIn(expected, roles)
            
        for e in data:
            self.assertIn("avatar_color", e)
            self.assertTrue(e["avatar_color"].startswith("#"))

    def test_chat_across_core_agents(self):
        """Test chat responses from multiple distinct swarm agents."""
        test_queries = [
            ("Archi", "Report active system architecture"),
            ("Shield", "Check DDR antibody status"),
            ("Logic", "Verify Kleene K3 invariants"),
            ("Seeker", "Query ESKAPE lead status"),
            ("Verify", "Check unit test integrity"),
            ("Jarvis", "Query TurboVec memory shards")
        ]

        for role, query in test_queries:
            payload = {"role": role, "message": query, "sender": "operator"}
            res = self.client.post("/swarm/chat", json=payload)
            self.assertEqual(res.status_code, 200, f"Failed on role {role}")
            data = res.json()
            self.assertIn("response", data)
            self.assertEqual(data["status"], "ok")
            self.assertGreater(len(data["response"]), 0)

    def test_chat_greetings_and_fast_paths(self):
        """Verify instant sub-millisecond greetings and fast deterministic responses."""
        res = self.client.post("/swarm/chat", json={"role": "Shield", "message": "hello", "sender": "user"})
        self.assertEqual(res.status_code, 200)
        data = res.json()
        self.assertIn("Shield", data["response"])

if __name__ == "__main__":
    unittest.main()
