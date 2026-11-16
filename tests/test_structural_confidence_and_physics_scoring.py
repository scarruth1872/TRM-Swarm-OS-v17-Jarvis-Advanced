"""
test_structural_confidence_and_physics_scoring.py — TRM Swarm OS v19.001 "Sovereign"
=====================================================================================
Unit & Invariant Tests for:
1. StructuralConfidenceFilter (AlphaFold pLDDT categorization, disordered loop pruning, pocket centering)
2. PhysicsScoringCorrector (Desolvation, electrostatics, H-bond geometry, rotatable bond penalties, Kd derivation)
"""

import sys
import os
import unittest
import math

PROJECT_ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from swarm_v2.core.structural_confidence_filter import (
    StructuralConfidenceFilter, ConfidenceBand
)
from swarm_v2.core.physics_scoring_correction import PhysicsScoringCorrector

class TestStructuralConfidenceFilter(unittest.TestCase):
    def setUp(self):
        self.filter = StructuralConfidenceFilter(plddt_threshold=70.0)

    def test_plddt_classification_bands(self):
        self.assertEqual(self.filter.classify_plddt(94.5), ConfidenceBand.VERY_HIGH)
        self.assertEqual(self.filter.classify_plddt(78.2), ConfidenceBand.CONFIDENT)
        self.assertEqual(self.filter.classify_plddt(58.0), ConfidenceBand.LOW)
        self.assertEqual(self.filter.classify_plddt(32.1), ConfidenceBand.VERY_LOW)

    def test_pocket_filtering_and_pruning(self):
        residues = [
            {"index": 1, "name": "MET", "plddt": 92.0, "coords_ca": (10.0, 10.0, 10.0)},
            {"index": 2, "name": "LEU", "plddt": 85.0, "coords_ca": (12.0, 10.5, 11.0)},
            {"index": 3, "name": "GLY", "plddt": 45.0, "coords_ca": (15.0, 12.0, 14.0)}, # Disordered
            {"index": 4, "name": "TYR", "plddt": 76.0, "coords_ca": (11.0, 9.0, 12.0)},
        ]
        
        result = self.filter.filter_residues("Target_TopoIV", residues)
        
        self.assertEqual(result.total_residues, 4)
        self.assertEqual(result.eligible_residues, 3)
        self.assertIn(3, result.pruned_disordered_indices)
        self.assertNotIn(3, result.retained_residue_indices)
        self.assertGreater(result.mean_plddt, 80.0)
        self.assertGreater(result.pocket_radius_angstroms, 4.0)


class TestPhysicsScoringCorrector(unittest.TestCase):
    def setUp(self):
        self.corrector = PhysicsScoringCorrector()

    def test_desolvation_and_electrostatics(self):
        desolv = self.corrector.compute_desolvation(nonpolar_sasa_buried_a2=150.0, polar_sasa_buried_a2=20.0)
        self.assertLess(desolv, 0.0)  # Net favorable due to large hydrophobic burial

        elec_attr = self.corrector.compute_electrostatics(formal_charge_ligand=-1.0, formal_charge_pocket=1.0, distance_angstroms=3.5)
        self.assertLess(elec_attr, 0.0)  # Favorable electrostatic salt bridge

    def test_full_complex_evaluation_trm_abx_219(self):
        breakdown = self.corrector.evaluate_complex(
            compound_id="TRM-ABX-219",
            target_id="A_baumannii_TopoIV",
            vina_base_dg_kcal=-9.82,
            num_rotatable_bonds=4,
            nonpolar_sasa_buried_a2=160.0,
            polar_sasa_buried_a2=25.0,
            formal_charge_ligand=-1.0,
            formal_charge_pocket=1.0,
            effective_charge_dist_a=3.2,
            hbond_angles_deg=[176.0, 172.0],
            hbond_distances_a=[2.80, 2.85]
        )

        self.assertEqual(breakdown.compound_id, "TRM-ABX-219")
        self.assertLess(breakdown.total_calibrated_dg_kcal, -9.0)
        self.assertGreater(breakdown.predicted_kd_nm, 0.0)
        self.assertLess(breakdown.predicted_kd_nm, 50.0)  # Strong nanomolar binder

if __name__ == "__main__":
    unittest.main()
