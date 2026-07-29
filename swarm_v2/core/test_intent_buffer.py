"""
Unit Tests for Predictive Intent Buffer
Phase 3: Intent Sync Latency Remediation Verification
"""

import unittest
import os
import json
import shutil
from unittest.mock import MagicMock, patch
from swarm_v2.core.intent_buffer import PredictiveIntentBuffer

class TestPredictiveIntentBuffer(unittest.TestCase):
    def setUp(self):
        # Create a temp directory for memory storage during test run
        self.test_dir = "swarm_v2_test_intent_memory"
        os.makedirs(self.test_dir, exist_ok=True)
        
        # Patch the SWARM_MEMORY_DIR environment variable
        self.env_patcher = patch.dict(os.environ, {"SWARM_MEMORY_DIR": self.test_dir})
        self.env_patcher.start()
        
        # Mock sys.modules['chromadb'] to avoid module-not-found errors
        import sys
        self.original_chroma = sys.modules.get('chromadb')
        self.mock_chroma_mod = MagicMock()
        sys.modules['chromadb'] = self.mock_chroma_mod
        
        # Reset file paths inside the class module
        import swarm_v2.core.intent_buffer
        swarm_v2.core.intent_buffer.GLOBAL_MEMORY_DIR = self.test_dir
        swarm_v2.core.intent_buffer.INTENT_DB_PATH = os.path.join(self.test_dir, "predictive_intents.json")
        
        self.buffer = PredictiveIntentBuffer()

    def tearDown(self):
        self.env_patcher.stop()
        
        # Restore original chromadb module if any
        import sys
        if self.original_chroma is None:
            sys.modules.pop('chromadb', None)
        else:
            sys.modules['chromadb'] = self.original_chroma
            
        if os.path.exists(self.test_dir):
            shutil.rmtree(self.test_dir)

    def test_seeding_and_loading(self):
        # Verify default seeding is populated
        self.assertGreater(len(self.buffer.intents), 0)
        
        # Verify specific role is present
        roles = [item["role"] for item in self.buffer.intents]
        self.assertIn("Orchestrator", roles)
        self.assertIn("Verify", roles)

    def test_record_new_intent(self):
        initial_count = len(self.buffer.intents)
        
        # Record a brand new intent
        self.buffer.record_intent("run cybersecurity diagnostic checks", "Shield", ["security", "diagnostic"])
        self.assertEqual(len(self.buffer.intents), initial_count + 1)
        
        # Check if saved to disk
        self.assertTrue(os.path.exists(os.path.join(self.test_dir, "predictive_intents.json")))
        
        # Record same intent again and verify frequency increases
        self.buffer.record_intent("run cybersecurity diagnostic checks", "Shield", ["security", "diagnostic"])
        self.assertEqual(len(self.buffer.intents), initial_count + 1)
        
        # Verify frequency
        matched = next(x for x in self.buffer.intents if x["query"] == "run cybersecurity diagnostic checks")
        self.assertEqual(matched["frequency"], 2)

    def test_predict_intent_cos_similarity(self):
        # Add distinct test intent
        self.buffer.record_intent("build responsive web interface layout", "Vision", ["ui", "layout"])
        
        # Try to predict with close token matching
        prediction = self.buffer.predict_intent("responsive web interface")
        self.assertEqual(prediction["role"], "Vision")
        self.assertGreater(prediction["confidence"], 0.4)
        self.assertEqual(prediction["source"], "tfidf_fallback")

    def test_predict_no_match(self):
        prediction = self.buffer.predict_intent("completely random unmatched word list")
        self.assertEqual(prediction["role"], "")
        self.assertEqual(prediction["confidence"], 0.0)

    @patch("swarm_v2.core.ses.get_ses")
    @patch("swarm_v2.experts.registry.get_expert_team")
    def test_prewarm_context(self, mock_get_expert_team, mock_get_ses):
        # Mock SES and Agent
        mock_ses = MagicMock()
        mock_ses.get_context.return_value = {
            "triples": [("UI", "hasLayout", "responsive")]
        }
        mock_get_ses.return_value = mock_ses
        
        mock_agent = MagicMock()
        mock_agent.memory = MagicMock()
        mock_get_expert_team.return_value = {"Vision": mock_agent}
        
        # Trigger pre-warming
        triples = self.buffer.prewarm_context("Vision", "web interface")
        
        # Assertions
        self.assertEqual(len(triples), 1)
        self.assertEqual(triples[0], ("UI", "hasLayout", "responsive"))
        mock_agent.memory.add_fact.assert_called_once()

if __name__ == "__main__":
    unittest.main()
