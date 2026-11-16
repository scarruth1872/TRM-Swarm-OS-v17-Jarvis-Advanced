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
