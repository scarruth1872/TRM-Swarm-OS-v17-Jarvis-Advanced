import unittest
import asyncio
from unittest.mock import MagicMock
from swarm_v2.core.self_healing_orchestrator import get_self_healing_orchestrator
from swarm_v2.core.agent_mesh import MeshNode

class TestSelfHealing(unittest.TestCase):
    def setUp(self):
        self.orchestrator = get_self_healing_orchestrator()
        # Reset state
        self.orchestrator._latency_windows.clear()
        self.orchestrator._degraded_agents.clear()
        self.orchestrator._reassignment_log.clear()
        self.orchestrator._cpu_loads.clear()
        self.orchestrator._task_counts.clear()
        self.orchestrator._error_counts.clear()

        # Mock mesh
        self.mock_mesh = MagicMock()
        self.orchestrator.bind_mesh(self.mock_mesh)

    def test_health_scoring_logic(self):
        # Fresh agent
        score = self.orchestrator.compute_health_score("Devo")
        self.assertAlmostEqual(score, 0.885, places=2)  # default values: error=0, cpu=0.3, latency=0.5 -> (1 - 0.5/8)*0.4 + 1.0*0.3 + 0.7*0.3 = 0.375 + 0.3 + 0.21 = 0.885

        # High CPU
        self.orchestrator.update_cpu_load("Devo", 0.9)
        score_high_cpu = self.orchestrator.compute_health_score("Devo")
        self.assertTrue(score_high_cpu < score)

    def test_reassignment(self):
        degraded = MeshNode(node_id="archi", name="Archi", role="Architect", specialties=["Design"])
        healthy = MeshNode(node_id="devo", name="Devo", role="Lead Developer", specialties=["Code"])
        
        self.mock_mesh.nodes = {"archi": degraded, "devo": healthy}
        
        engine_team = {"Architect": MagicMock(), "Lead Developer": MagicMock()}
        
        self.orchestrator._reassign_role("Architect", engine_team)
        self.assertIn("Design", healthy.specialties)
        self.assertEqual(degraded.status, "degraded")
        self.assertIn("Architect", self.orchestrator._degraded_agents)

    def test_restoration(self):
        degraded = MeshNode(node_id="archi", name="Archi", role="Architect", specialties=["Design"])
        healthy = MeshNode(node_id="devo", name="Devo", role="Lead Developer", specialties=["Code", "Design"])
        
        self.mock_mesh.nodes = {"archi": degraded, "devo": healthy}
        
        self.orchestrator._degraded_agents["Architect"] = {
            "transferred_to": "Lead Developer",
            "added_specialties": ["Design"],
            "recovery_streak": 0
        }
        
        self.orchestrator._restore_agent("Architect")
        self.assertNotIn("Design", healthy.specialties)
        self.assertEqual(degraded.status, "online")
        self.assertNotIn("Architect", self.orchestrator._degraded_agents)

if __name__ == '__main__':
    unittest.main()
