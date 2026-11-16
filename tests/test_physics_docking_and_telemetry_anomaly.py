"""
test_physics_docking_and_telemetry_anomaly.py — TRM Swarm OS v19.001 "Sovereign"
================================================================================
Comprehensive Tests for:
1. Priority 4: Physics-Based Free Energy Scoring Refinement (Schrödinger-style ΔG_desolv, ΔG_elec, ΔG_hbond)
2. Priority 5: Telemetry Anomaly Baselining (Darktrace-style Rolling Z-Score & Autonomous Mitigation)
"""

import sys
import os
import unittest
import math
import time

PROJECT_ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from swarm_v2.core.physics_scoring_correction import PhysicsScoringCorrector
from swarm_v2.core.telemetry_anomaly_baseliner import (
    TelemetryAnomalyBaseliner, ThreatLevel
)

class TestPhysicsScoringPipeline(unittest.TestCase):
    def setUp(self):
        self.corrector = PhysicsScoringCorrector()

    def test_desolvation_and_charge_screening(self):
        # Favorable hydrophobic burial: large non-polar SASA, small polar SASA
        desolv = self.corrector.compute_desolvation(nonpolar_sasa_buried_a2=180.0, polar_sasa_buried_a2=15.0)
        self.assertLess(desolv, 0.0) # Should be negative (favorable)

        # Salt bridge attraction
        elec = self.corrector.compute_electrostatics(formal_charge_ligand=-1.0, formal_charge_pocket=1.0, distance_angstroms=2.9)
        self.assertLess(elec, 0.0)

    def test_recalibrated_docking_lead_abx_219(self):
        result = self.corrector.evaluate_complex(
            compound_id="TRM-ABX-219",
            target_id="A_baumannii_TopoIV",
            vina_base_dg_kcal=-9.82,
            num_rotatable_bonds=4,
            nonpolar_sasa_buried_a2=175.0,
            polar_sasa_buried_a2=20.0,
            formal_charge_ligand=-1.0,
            formal_charge_pocket=1.0,
            effective_charge_dist_a=3.1,
            hbond_angles_deg=[178.0, 174.0],
            hbond_distances_a=[2.78, 2.82]
        )
        self.assertLess(result.total_calibrated_dg_kcal, -10.0)
        self.assertLess(result.predicted_kd_nm, 20.0) # Confirmed high-potency nanomolar binder


class TestTelemetryAnomalyBaseliner(unittest.TestCase):
    def setUp(self):
        self.baseliner = TelemetryAnomalyBaseliner(window_size=50, warning_z=2.0, critical_z=3.0)
        # Seed normal baseline on msg_rate (mean ~ 100 msgs/s, std ~ 5)
        for i in range(50):
            val = 100.0 + (i % 5) - 2.0
            self.baseliner.observe("msg_rate_mps", val)

    def test_normal_telemetry_flow(self):
        event = self.baseliner.observe("msg_rate_mps", 101.5)
        self.assertEqual(event.threat_level, ThreatLevel.NORMAL)
        self.assertIsNone(event.mitigation_action)

    def test_warning_level_drift(self):
        # Introduce a modest drift (Z ~ 2.47)
        event = self.baseliner.observe("msg_rate_mps", 103.5)
        self.assertEqual(event.threat_level, ThreatLevel.ELEVATED_WARNING)

    def test_critical_anomaly_spike_and_mitigation(self):
        # Huge spike (e.g. 500 msgs/sec, Z > 10)
        event = self.baseliner.observe("msg_rate_mps", 550.0)
        self.assertEqual(event.threat_level, ThreatLevel.CRITICAL_ANOMALY)
        self.assertEqual(event.mitigation_action, "RATE_LIMITER_ENGAGED_CAP_500_MPS")
        self.assertGreater(len(self.baseliner.mitigation_history), 0)

    def test_system_status_reporting(self):
        status = self.baseliner.get_system_telemetry_status()
        self.assertIn("msg_rate_mps", status["channels"])
        self.assertGreater(status["channels"]["msg_rate_mps"]["sample_count"], 40)

if __name__ == "__main__":
    unittest.main()
