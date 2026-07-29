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
