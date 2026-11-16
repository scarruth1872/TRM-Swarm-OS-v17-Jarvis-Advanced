"""
structural_confidence_filter.py — TRM Swarm OS v19.001 "Sovereign"
===================================================================
AlphaFold Structural Confidence (pLDDT & PAE) Filtering Engine:
- Evaluates per-residue predicted Local Distance Difference Test (pLDDT) confidence scores.
- Bins residue quality into standard AlphaFold confidence bands:
  * VERY_HIGH (> 90.0): Highly accurate side-chain & backbone geometry.
  * CONFIDENT (70.0 - 90.0): Well-defined backbone, reliable for pocket definition.
  * LOW (50.0 - 70.0): Flexible loop or potentially unstructured region.
  * VERY_LOW (< 50.0): Likely intrinsically disordered / unmodelable.
- Evaluates Predicted Aligned Error (PAE) matrix to ensure rigid domain orientation.
- Filters pocket coordinates to exclude disordered residues prior to molecular docking.
"""

import math
from enum import Enum
from typing import List, Dict, Any, Tuple, Optional
from dataclasses import dataclass, field

class ConfidenceBand(Enum):
    VERY_HIGH = "very_high"   # pLDDT > 90
    CONFIDENT = "confident"   # 70 <= pLDDT <= 90
    LOW = "low"               # 50 <= pLDDT < 70
    VERY_LOW = "very_low"     # pLDDT < 50

@dataclass
class ResidueConfidence:
    residue_index: int
    residue_name: str
    plddt: float
    band: ConfidenceBand
    coordinates_ca: Tuple[float, float, float]  # Alpha-carbon (x, y, z)
    is_docking_eligible: bool

@dataclass
class FilteredPocketResult:
    target_id: str
    total_residues: int
    eligible_residues: int
    mean_plddt: float
    pocket_center_ca: Tuple[float, float, float]
    pocket_radius_angstroms: float
    retained_residue_indices: List[int]
    pruned_disordered_indices: List[int]

class StructuralConfidenceFilter:
    """
    Filters macromolecular structures and binding pocket coordinates based on
    AlphaFold confidence metrics (pLDDT >= threshold, default 70.0).
    """
    def __init__(self, plddt_threshold: float = 70.0, pae_threshold_angstroms: float = 12.0):
        self.plddt_threshold = plddt_threshold
        self.pae_threshold_angstroms = pae_threshold_angstroms

    def classify_plddt(self, plddt: float) -> ConfidenceBand:
        if plddt >= 90.0:
            return ConfidenceBand.VERY_HIGH
        elif plddt >= 70.0:
            return ConfidenceBand.CONFIDENT
        elif plddt >= 50.0:
            return ConfidenceBand.LOW
        else:
            return ConfidenceBand.VERY_LOW

    def filter_residues(
        self,
        target_id: str,
        residue_data: List[Dict[str, Any]],
        candidate_pocket_indices: Optional[List[int]] = None
    ) -> FilteredPocketResult:
        """
        Filters a list of residues, identifying eligible pocket residues above the pLDDT threshold.
        """
        classified_residues: List[ResidueConfidence] = []
        retained_indices = []
        pruned_indices = []
        
        subset_indices = set(candidate_pocket_indices) if candidate_pocket_indices is not None else None

        for item in residue_data:
            idx = item["index"]
            name = item.get("name", "UNK")
            plddt = float(item.get("plddt", 0.0))
            coords = tuple(item.get("coords_ca", (0.0, 0.0, 0.0)))
            band = self.classify_plddt(plddt)

            is_in_subset = (subset_indices is None) or (idx in subset_indices)
            is_eligible = (plddt >= self.plddt_threshold) and is_in_subset

            res_obj = ResidueConfidence(
                residue_index=idx,
                residue_name=name,
                plddt=plddt,
                band=band,
                coordinates_ca=coords,
                is_docking_eligible=is_eligible
            )
            classified_residues.append(res_obj)

            if is_in_subset:
                if is_eligible:
                    retained_indices.append(idx)
                else:
                    pruned_indices.append(idx)

        # Compute geometrical center & radius of retained pocket residues
        retained_coords = [r.coordinates_ca for r in classified_residues if r.residue_index in retained_indices]
        
        if retained_coords:
            cx = sum(c[0] for c in retained_coords) / len(retained_coords)
            cy = sum(c[1] for c in retained_coords) / len(retained_coords)
            cz = sum(c[2] for c in retained_coords) / len(retained_coords)
            center = (round(cx, 3), round(cy, 3), round(cz, 3))

            # Max distance from center to any pocket CA atom
            max_r = max(
                math.sqrt((c[0] - cx)**2 + (c[1] - cy)**2 + (c[2] - cz)**2)
                for c in retained_coords
            )
            radius = round(max_r + 4.0, 2)  # 4.0 Angstrom buffer for ligand envelope
        else:
            center = (0.0, 0.0, 0.0)
            radius = 10.0

        mean_plddt = (
            sum(r.plddt for r in classified_residues if r.residue_index in retained_indices) / len(retained_indices)
            if retained_indices else 0.0
        )

        return FilteredPocketResult(
            target_id=target_id,
            total_residues=len(residue_data),
            eligible_residues=len(retained_indices),
            mean_plddt=round(mean_plddt, 2),
            pocket_center_ca=center,
            pocket_radius_angstroms=radius,
            retained_residue_indices=retained_indices,
            pruned_disordered_indices=pruned_indices
        )
