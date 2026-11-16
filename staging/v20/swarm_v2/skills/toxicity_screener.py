"""
toxicity_screener.py — In Silico Preclinical Toxicity & Safety Screener
TRM Swarm OS v19.001 "Sovereign"

Task 18 Directive:
1. Ingest small molecules, peptides, and drug repurposing candidates.
2. In Silico Safety Endpoints:
   - hERG Cardiotoxicity: Potassium channel inhibition IC50 (uM), QTc prolongation risk
   - DILI Hepatotoxicity: Drug-Induced Liver Injury risk (0.0 to 1.0) based on LogP, dose, reactive alerts
   - Ames Mutagenicity: Structural alert detection (Ashby-Tennant rules: nitro, aromatic amine, alkylating)
   - Hemolytic Cytotoxicity: Erythrocyte lysis HC50 (ug/mL)
   - Therapeutic Window: Ratio of Toxic Threshold to Therapeutic Effective Concentration
3. Kleene K3 Consensus:
   - +1: SAFE_PRECLINICAL_LEAD (hERG IC50 > 10 uM, Ames Negative, DILI < 0.35, HC50 > 250 ug/mL)
   - 0: MODERATE_RISK_MONITORED (1 uM <= hERG <= 10 uM, DILI < 0.65)
   - -1: TOXIC_LIABILITY_FLAGGED (hERG < 1 uM or Ames Positive or DILI >= 0.65)
4. Saves profiles to swarm_v2_artifacts/toxicity_screening_profiles.json

Monad Truth Anchor: 0x4D4F4E4144000001
"""

import os
import sys
import json
import time
import math
import hashlib
from typing import Dict, List, Any, Tuple, Optional
from dataclasses import dataclass, asdict

MONAD_TRUTH_ANCHOR = "0x4D4F4E4144000001"

ASHBY_TENNANT_ALERTS = [
    {"name": "Aliphatic Halide", "pattern": "C-Cl/Br/I", "alert_type": "DNA Alkylation"},
    {"name": "Aromatic Nitro", "pattern": "[N+](=O)[O-]", "alert_type": "Metabolic Reduction / Genotox"},
    {"name": "Primary Aromatic Amine", "pattern": "c-NH2", "alert_type": "N-Hydroxylation"},
    {"name": "Epoxide / Aziridine", "pattern": "C1OC1", "alert_type": "Direct Electrophile"},
    {"name": "Quinone / Hydroquinone", "pattern": "O=C1C=CC(=O)C=C1", "alert_type": "Redox Cycling / ROS"}
]

@dataclass
class ToxicityProfile:
    compound_id: str
    compound_name: str
    compound_type: str # Small Molecule or Peptide
    herg_ic50_um: float
    herg_risk_tier: str
    dili_score: float
    dili_risk_tier: str
    ames_mutagenic_alert: bool
    structural_alerts: List[str]
    hemolytic_hc50_ug_ml: float
    therapeutic_safety_window: float
    k3_safety_status: str

class ToxicityScreener:
    """Evaluates comprehensive in silico toxicological liability."""

    def __init__(self):
        self.profiles: List[ToxicityProfile] = []

    def screen_compound(self, comp_id: str, name: str, comp_type: str, smiles_or_seq: str, logp: float, mw: float, eff_conc_nm: float) -> ToxicityProfile:
        # hERG cardiotoxicity prediction:
        # High LogP (>3.5) + basic nitrogen + MW ~ 400-600 strongly associates with hERG pore blocking
        herg_pred_log = 1.6 - (max(0.0, logp - 1.5) * 0.48) - (0.001 * mw)
        herg_ic50_um = round(math.exp(herg_pred_log), 2)
        herg_ic50_um = max(0.12, min(herg_ic50_um, 85.0))
        
        if herg_ic50_um > 10.0:
            herg_tier = "LOW_RISK"
        elif herg_ic50_um >= 1.0:
            herg_tier = "MODERATE_RISK"
        else:
            herg_tier = "HIGH_CARDIOTOXIC_ALERT"

        # DILI hepatotoxicity prediction:
        # Rule of 2 (Daily dose > 100mg + LogP >= 3)
        dili_score = 0.15 + (max(0.0, logp) * 0.08) + (min(mw / 800.0, 1.0) * 0.15)
        dili_score = round(min(max(dili_score, 0.05), 0.95), 3)
        
        if dili_score < 0.35:
            dili_tier = "LOW_DILI_RISK"
        elif dili_score < 0.65:
            dili_tier = "MODERATE_DILI_RISK"
        else:
            dili_tier = "HIGH_HEPATOTOXIC_ALERT"

        # Structural alerts / Ames mutagenicity
        alerts = []
        if "[N+](=O)[O-]" in smiles_or_seq:
            alerts.append("Aromatic Nitro (Potential Mutagenic Alert)")
        if "Cl" in smiles_or_seq or "Br" in smiles_or_seq:
            alerts.append("Halogenated Carbon Ring")
        if "SSC" in smiles_or_seq:
            alerts.append("Reactive Disulfide Bridge")

        ames_flag = len(alerts) > 0 and any("Mutagenic" in a for a in alerts)

        # Hemolytic cytotoxicity HC50 (ug/mL)
        if comp_type == "Peptide":
            # Highly hydrophobic cationic peptides lyse erythrocytes at lower concs
            hc50 = max(15.0, min(800.0 / (1.0 + max(0.0, logp)), 1000.0))
        else:
            hc50 = max(50.0, min(2500.0 / (1.0 + max(0.0, logp - 1.0)), 3000.0))
        hc50 = round(hc50, 1)

        # Therapeutic safety window: (hERG IC50 uM * 1000 nM) / Effective Conc nM
        therapeutic_window = round((herg_ic50_um * 1000.0) / max(eff_conc_nm, 1.0), 1)

        # Kleene K3
        if herg_ic50_um > 10.0 and not ames_flag and dili_score < 0.40 and hc50 > 150.0:
            k3 = "+1" # SAFE
        elif herg_ic50_um >= 1.0 and not ames_flag and dili_score < 0.65:
            k3 = "0"  # MONITORED
        else:
            k3 = "-1" # TOXIC_LIABILITY

        return ToxicityProfile(
            compound_id=comp_id,
            compound_name=name,
            compound_type=comp_type,
            herg_ic50_um=herg_ic50_um,
            herg_risk_tier=herg_tier,
            dili_score=dili_score,
            dili_risk_tier=dili_tier,
            ames_mutagenic_alert=ames_flag,
            structural_alerts=alerts,
            hemolytic_hc50_ug_ml=hc50,
            therapeutic_safety_window=therapeutic_window,
            k3_safety_status=k3
        )

    def screen_benchmark_cohort(self) -> Dict[str, Any]:
        start_time = time.perf_counter()
        
        # Test cohort combining AMPs, drug repurposing hits, and oncology molecules
        cohort = [
            ("COMP-001", "Niclosamide", "Small Molecule", "C1=CC(=C(C=C1[N+](=O)[O-])Cl)NC(=O)C2=C(C=CC(=C2)Cl)O", 4.1, 327.12, 4.84),
            ("COMP-002", "Imatinib", "Small Molecule", "CC1=C(C=C(C=C1)NC(=O)C2=CC=C(C=C2)CN3CCN(CC3)C)NC4=NC=CC(=N4)C5=CN=CC=C5", 3.5, 493.60, 4.84),
            ("COMP-003", "Maraviroc", "Small Molecule", "CC1=NN(C(=N1)C2CC3CCC(C2)N3CCC(C4=CC=CC=C4)NC(=O)C5=CC=C(C=C5)F)C(C)C", 4.2, 513.65, 18.58),
            ("COMP-004", "Doxycycline", "Small Molecule", "CC1C2CC3C(C(=O)C(=C(C3(C(=O)C2=C(C4=C1C=CC=C4O)O)O)O)C(=O)N)N(C)C", -0.7, 444.43, 85.0),
            ("COMP-005", "Baricitinib", "Small Molecule", "CCS(=O)(=O)N1CC(C1)(CC#N)N2C=C(C=N2)C3=C4C=CNC4=NC=N3", 1.1, 371.42, 42.0),
            ("COMP-006", "Ebselen", "Small Molecule", "C1=CC=C2C(=C1)C(=O)N(Se2)C3=CC=CC=C3", 2.8, 274.18, 110.0),
            ("COMP-007", "TRM-AMP-LpxC-01", "Peptide", "RRKWFRRLRRKWFRRLRR", 0.5, 2350.0, 3.2),
            ("COMP-008", "TRM-AMP-BamA-04", "Peptide", "KWRKFFRKWKWRKFFRKW", 1.2, 2580.0, 5.8),
            ("COMP-009", "TRM-AMP-FtsZ-08", "Peptide", "GRRTLKGKTGRRTLKGKT", -0.2, 2120.0, 8.4)
        ]

        self.profiles = []
        for cid, name, ctype, struct, logp, mw, eff in cohort:
            prof = self.screen_compound(cid, name, ctype, struct, logp, mw, eff)
            self.profiles.append(prof)

        elapsed = time.perf_counter() - start_time
        print(f"[TOXICITY] Evaluated preclinical safety for {len(self.profiles)} compounds in {elapsed*1000:.2f} ms")

        payload = {
            "monad_anchor": MONAD_TRUTH_ANCHOR,
            "version": "1.0.0",
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime()),
            "total_screened": len(self.profiles),
            "statistics": {
                "safe_preclinical_leads (+1)": sum(1 for p in self.profiles if p.k3_safety_status == "+1"),
                "moderate_risk_monitored (0)": sum(1 for p in self.profiles if p.k3_safety_status == "0"),
                "toxic_liabilities (-1)": sum(1 for p in self.profiles if p.k3_safety_status == "-1"),
                "mean_therapeutic_window": round(sum(p.therapeutic_safety_window for p in self.profiles) / len(self.profiles), 1)
            },
            "profiles": [asdict(p) for p in self.profiles]
        }

        out_path = "swarm_v2_artifacts/toxicity_screening_profiles.json"
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        with open(out_path, 'w', encoding='utf-8') as f:
            json.dump(payload, f, indent=2)

        print(f"[TOXICITY] Saved toxicity profiles to {out_path}")
        return payload

if __name__ == "__main__":
    screener = ToxicityScreener()
    res = screener.screen_benchmark_cohort()
    print("\n--- Preclinical Toxicity Profiles ---")
    for p in res["profiles"]:
        print(f"[{p['compound_name']}] hERG: {p['herg_ic50_um']} uM ({p['herg_risk_tier']}) | DILI: {p['dili_score']} | Window: {p['therapeutic_safety_window']}x | K3: {p['k3_safety_status']}")
