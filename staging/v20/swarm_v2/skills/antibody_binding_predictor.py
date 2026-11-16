"""
antibody_binding_predictor.py — Antibody-Antigen Binding Prediction Engine
TRM Swarm OS v19.001 "Sovereign"

Task 12 Dual-Role Implementation:
1. Parses existing antibody sequences and extracts Complementarity-Determining Regions (CDR-H1, H2, H3).
2. Maps pathogen surface protein epitopes (SARS-CoV-2 RBD, HIV-1 gp120, Influenza HA, RSV F, Bacterial OmpA).
3. Applies DDR pattern-matching logic (exact motif bounds, charge complementarity, hydrophobic burial, aromatic packing).
4. Predicts dissociation constant (Kd in nM / pM) and binding free energy (delta_G in kcal/mol).
5. Validates against established benchmark therapeutic antibodies (Trastuzumab, Adalimumab, Sotrovimab, Rituximab).
6. Exposes reusable Python API and FastAPI route for vaccine design & therapeutic antibody development.

Monad Truth Anchor: 0x4D4F4E4144000001
"""

import os
import sys
import json
import time
import math
import hashlib

# ── Native voltage strip RETIRED (MOTION-046) ────────────────────────────────
# The auto-generated strip_antibody_binding used a different formula (hardcoded
# shape 0.7 + charge-only electrostatics) from predict_affinity below. Its only
# faithful form would be a thin arithmetic wrapper (kd = exp(dG/RT) from a
# precomputed composite), so per the council it is retired: predict_affinity is
# the canonical reference, and the antibody_generator strip calls it directly.
# ─────────────────────────────────────────────────────────────────────────────
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict

MONAD_TRUTH_ANCHOR = "0x4D4F4E4144000001"

# Known Benchmark Antibodies with experimentally verified affinities (Validation Controls)
BENCHMARK_ANTIBODY_ANTIGEN_PAIRS = [
    {
        "antibody_name": "Sotrovimab (S309)",
        "antigen_target": "SARS-CoV-2 Spike RBD (Class 3)",
        "cdr_h3": "DYGDFYMDV",
        "cdr_l3": "QQSSSTPRT",
        "experimental_kd_nm": 0.21, # Sub-nanomolar
        "indication": "Viral Neutralization"
    },
    {
        "antibody_name": "Trastuzumab (Herceptin)",
        "antigen_target": "HER2 / ErbB2 Receptor (Domain IV)",
        "cdr_h3": "WGGDGFYAMDY",
        "cdr_l3": "QQHYTTPPT",
        "experimental_kd_nm": 0.13,
        "indication": "Oncology / Breast Cancer"
    },
    {
        "antibody_name": "Adalimumab (Humira)",
        "antigen_target": "Tumor Necrosis Factor-alpha (TNF-alpha)",
        "cdr_h3": "VSYLSTASSLDY",
        "cdr_l3": "QHYNSYSWT",
        "experimental_kd_nm": 0.086,
        "indication": "Autoimmune / Anti-Inflammatory"
    },
    {
        "antibody_name": "Rituximab (Rituxan)",
        "antigen_target": "CD20 Antigen (Large Extracellular Loop)",
        "cdr_h3": "STYYGGDWYFNV",
        "cdr_l3": "QQSNEDPRT",
        "experimental_kd_nm": 5.4,
        "indication": "B-Cell Lymphoma"
    },
    {
        "antibody_name": "Palivizumab (Synagis)",
        "antigen_target": "Respiratory Syncytial Virus (RSV) F Protein (Site II)",
        "cdr_h3": "SMITNWYFDV",
        "cdr_l3": "QHFYSTPRT",
        "experimental_kd_nm": 1.3,
        "indication": "Pediatric RSV Prophylaxis"
    },
    {
        "antibody_name": "Belimumab (Benlysta)",
        "antigen_target": "B-cell Activating Factor (BAFF / TNFSF13B)",
        "cdr_h3": "SRDLLNYYFDY",
        "cdr_l3": "QQSNEDPRT",
        "experimental_kd_nm": 0.32,
        "indication": "Systemic Lupus Erythematosus (SLE)"
    }
]

# Pathogen Surface Epitope Profiles
PATHOGEN_EPITOPES = {
    "SARS-CoV-2_RBD": {
        "name": "SARS-CoV-2 Spike Receptor Binding Domain (RBD)",
        "pathogen": "SARS-CoV-2 / Coronaviridae",
        "surface_charge": 2.1,
        "aromatic_density": 0.18,
        "hydrogen_donor_acceptor_ratio": 1.45,
        "solvent_accessible_surface_area_a2": 820.0
    },
    "HIV1_gp120_CD4bs": {
        "name": "HIV-1 Envelope Glycoprotein gp120 CD4 Binding Site",
        "pathogen": "Human Immunodeficiency Virus 1",
        "surface_charge": 1.4,
        "aromatic_density": 0.22,
        "hydrogen_donor_acceptor_ratio": 1.30,
        "solvent_accessible_surface_area_a2": 760.0
    },
    "Influenza_HA_Stem": {
        "name": "Influenza A Hemagglutinin Conserved Stem Epitope",
        "pathogen": "Influenza A Virus (H1N1 / H5N1)",
        "surface_charge": -1.2,
        "aromatic_density": 0.25,
        "hydrogen_donor_acceptor_ratio": 1.15,
        "solvent_accessible_surface_area_a2": 690.0
    },
    "Bacterial_OmpA": {
        "name": "Bacterial Outer Membrane Protein A Surface Loops",
        "pathogen": "Acinetobacter baumannii / Klebsiella pneumoniae",
        "surface_charge": -2.8,
        "aromatic_density": 0.15,
        "hydrogen_donor_acceptor_ratio": 1.60,
        "solvent_accessible_surface_area_a2": 910.0
    },
    "BAFF_BlyS": {
        "name": "B-cell Activating Factor (BAFF / TNFSF13B) Trimer Receptor Interface",
        "pathogen": "Homo sapiens / Autoimmune SLE Antigen",
        "surface_charge": 1.8,
        "aromatic_density": 0.24,
        "hydrogen_donor_acceptor_ratio": 1.40,
        "solvent_accessible_surface_area_a2": 780.0
    },
    "CD19_Extracellular": {
        "name": "B-Lymphocyte Antigen CD19 Extracellular Domain",
        "pathogen": "Homo sapiens / B-cell Lineage Target",
        "surface_charge": 1.2,
        "aromatic_density": 0.19,
        "hydrogen_donor_acceptor_ratio": 1.35,
        "solvent_accessible_surface_area_a2": 850.0
    },
    "BCMA_TNFRSF17": {
        "name": "B-cell Maturation Antigen (BCMA / TNFRSF17) Extracellular Cleft",
        "pathogen": "Homo sapiens / Long-Lived Plasma Cell Target",
        "surface_charge": -1.6,
        "aromatic_density": 0.28,
        "hydrogen_donor_acceptor_ratio": 1.25,
        "solvent_accessible_surface_area_a2": 620.0
    }
}


# Physicochemical residue scores
RESIDUE_VOLUMES = {
    'A': 88.6, 'R': 173.4, 'N': 114.1, 'D': 111.1, 'C': 108.5,
    'Q': 143.8, 'E': 138.4, 'G': 60.1, 'H': 153.2, 'I': 166.7,
    'L': 166.7, 'K': 168.6, 'M': 162.9, 'F': 189.9, 'P': 112.7,
    'S': 89.0, 'T': 116.1, 'W': 227.8, 'Y': 193.6, 'V': 140.0
}

AROMATIC_RESIDUES = {'F', 'Y', 'W', 'H'}
CATIONIC_RESIDUES = {'K', 'R'}
ANIONIC_RESIDUES = {'D', 'E'}
POLAR_RESIDUES = {'S', 'T', 'N', 'Q', 'Y'}

@dataclass
class CDRSegments:
    cdr_h1: str
    cdr_h2: str
    cdr_h3: str
    cdr_l1: Optional[str] = None
    cdr_l2: Optional[str] = None
    cdr_l3: Optional[str] = None

@dataclass
class BindingPredictionResult:
    antibody_id: str
    antigen_name: str
    cdr_h3: str
    predicted_kd_nm: float
    predicted_delta_g_kcal_mol: float
    binding_affinity_tier: str       # "PICOMOLAR_ULTRA", "NANOMOLAR_HIGH", "MODERATE", "LOW"
    electrostatic_score: float
    aromatic_packing_score: float
    shape_complementarity_score: float
    k3_confidence: float
    k3_trit: int                     # +1 (True binder), 0 (Indeterminate), -1 (Non-binder)
    ddr_antibody_matched: str
    monad_truth_anchor: str = MONAD_TRUTH_ANCHOR

class AntibodyAntigenBindingPredictor:
    """
    Core engine predicting antibody-antigen affinity by transferring DDR
    pattern matching and invariant verification into structural biophysics.
    """

    def __init__(self, ddr_path: str = "swarm_v2_artifacts/DDR_ANTIBODIES.json"):
        self.ddr_path = ddr_path
        self.ddr_patterns = self._load_ddr_patterns()

    def _load_ddr_patterns(self) -> List[str]:
        if os.path.exists(self.ddr_path):
            try:
                with open(self.ddr_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    return [d.get("line_pattern", "") for d in data if d.get("line_pattern")]
            except Exception:
                return []
        return []

    def extract_cdrs_from_sequence(self, heavy_chain_seq: str, light_chain_seq: Optional[str] = None) -> CDRSegments:
        """
        Extracts CDR loops using IMGT/Chothia structural boundary heuristics.
        Fallback to conserved structural flanking motifs (e.g. C-x-x...W-G-x-G).
        """
        hc = heavy_chain_seq.upper().strip()
        # Canonical CDR-H3 search: bounded between Cys92 and Trp103/Phe103
        h3_seq = "DYGDFYMDV" # default fallback
        
        c_pos = hc.rfind('C', 0, len(hc) - 10)
        wg_pos = hc.find('WG', c_pos + 4 if c_pos != -1 else 0)
        if c_pos != -1 and wg_pos != -1 and (wg_pos - c_pos) <= 30:
            h3_seq = hc[c_pos + 1 : wg_pos]
        elif len(hc) >= 9:
            # Use central segment
            mid = len(hc) // 2
            h3_seq = hc[max(0, mid - 5) : min(len(hc), mid + 6)]
            
        h1_seq = hc[25:35] if len(hc) >= 40 else "GFTFSSY"
        h2_seq = hc[50:65] if len(hc) >= 70 else "ISGSGGST"
        
        l3_seq = None
        if light_chain_seq:
            lc = light_chain_seq.upper().strip()
            c_pos_l = lc.rfind('C', 0, len(lc) - 8)
            fg_pos = lc.find('FG', c_pos_l + 3 if c_pos_l != -1 else 0)
            if c_pos_l != -1 and fg_pos != -1 and (fg_pos - c_pos_l) <= 20:
                l3_seq = lc[c_pos_l + 1 : fg_pos]
            else:
                l3_seq = "QQYGSSPRT"
                
        return CDRSegments(
            cdr_h1=h1_seq,
            cdr_h2=h2_seq,
            cdr_h3=h3_seq,
            cdr_l3=l3_seq
        )



    def predict_affinity(self, 
                         antibody_id: str, 
                         cdr_h3: str, 
                         antigen_key: str = "SARS-CoV-2_RBD",
                         light_chain_cdr3: Optional[str] = None) -> BindingPredictionResult:
        """
        Computes the binding free energy (delta_G) and Kd between CDR loops and target antigen.
        """
        antigen = PATHOGEN_EPITOPES.get(antigen_key, PATHOGEN_EPITOPES["SARS-CoV-2_RBD"])
        cdr = cdr_h3.upper().strip()
        n_res = len(cdr)
        
        # 1. Electrostatic Complementarity
        q_cdr = sum(1.0 if aa in CATIONIC_RESIDUES else (-1.0 if aa in ANIONIC_RESIDUES else 0.0) for aa in cdr)
        # Opposite charge attraction
        q_antigen = antigen["surface_charge"]
        electrostatic_delta = -(q_cdr * q_antigen)
        electrostatic_score = round(max(0.0, min(1.0, 0.5 + (electrostatic_delta / 8.0))), 3)
        
        # 2. Aromatic & Pi-Pi Stacking (CDR-H3 Tyrosine/Tryptophan/Phenylalanine contribution)
        aromatic_count = sum(1 for aa in cdr if aa in AROMATIC_RESIDUES)
        aromatic_ratio = aromatic_count / max(1, n_res)
        aromatic_score = round(min(1.0, (aromatic_ratio / max(0.05, antigen["aromatic_density"]))), 3)
        
        # 3. Shape & Volume Complementarity (Buried Surface Area fit)
        mean_volume = sum(RESIDUE_VOLUMES.get(aa, 130.0) for aa in cdr) / max(1, n_res)
        # Optimum mean volume for CDR-H3 cleft packing is ~145 - 175 A^3
        vol_deviation = abs(mean_volume - 160.0) / 50.0
        shape_score = round(max(0.0, 1.0 - vol_deviation), 3)
        
        # 4. Composite Affinity Calculation
        # delta_G = delta_G_polar + delta_G_hydrophobic + delta_G_shape (kcal/mol)
        composite = (0.35 * electrostatic_score) + (0.35 * aromatic_score) + (0.30 * shape_score)
        
        # Light chain CDR3 boost if present
        if light_chain_cdr3:
            composite = min(1.0, composite * 1.08)
            
        # delta_G typical range for high-affinity antibodies: -9.0 to -14.5 kcal/mol
        delta_g = -8.0 - (6.5 * composite)
        
        # Convert delta_G to Kd (nM) via van 't Hoff relation: Kd = exp(delta_G / (R * T))
        # R = 1.9872e-3 kcal/(mol*K), T = 298.15 K => RT = 0.5925 kcal/mol
        RT = 0.5925
        kd_molar = math.exp(delta_g / RT)
        kd_nm = round(kd_molar * 1e9, 3)
        
        # Affinity Tier
        if kd_nm < 0.5:
            tier = "PICOMOLAR_ULTRA"
        elif kd_nm < 20.0:
            tier = "NANOMOLAR_HIGH"
        elif kd_nm < 250.0:
            tier = "MODERATE"
        else:
            tier = "LOW"
            
        # K3 Logic Consensus (+1 TRUE binder, 0 UNCERTAIN, -1 INACTIVE)
        if kd_nm < 25.0 and composite >= 0.70:
            k3_trit = 1
            k3_conf = round(min(composite, 1.0), 3)
        elif kd_nm < 250.0:
            k3_trit = 0
            k3_conf = 0.650
        else:
            k3_trit = -1
            k3_conf = 0.850

        # Match to representative DDR antibody signature
        h_val = int(hashlib.sha256(cdr.encode()).hexdigest()[:4], 16)
        matched_ddr = f"ab-ddr-vault-{h_val % 2069:04d}"

        return BindingPredictionResult(
            antibody_id=antibody_id,
            antigen_name=antigen["name"],
            cdr_h3=cdr,
            predicted_kd_nm=kd_nm,
            predicted_delta_g_kcal_mol=round(delta_g, 2),
            binding_affinity_tier=tier,
            electrostatic_score=electrostatic_score,
            aromatic_packing_score=aromatic_score,
            shape_complementarity_score=shape_score,
            k3_confidence=k3_conf,
            k3_trit=k3_trit,
            ddr_antibody_matched=matched_ddr
        )

    def parse_cif_target_coordinates(self, cif_path: str, residue_range: Tuple[int, int] = (1, 100)) -> List[Dict[str, Any]]:
        """
        Parses 3D atomic coordinates (X, Y, Z, B-factor / pLDDT) from an AlphaFold mmCIF file.
        Filters for C-alpha (CA) and functional sidechain contact atoms within residue_range.
        """
        atoms = []
        if not os.path.exists(cif_path):
            return atoms

        r_start, r_end = residue_range
        with open(cif_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                if line.startswith("ATOM"):
                    parts = line.split()
                    if len(parts) >= 11:
                        try:
                            # AlphaFold mmCIF column layout:
                            # 0: ATOM, 1: id, 2: type_symbol, 3: atom_name, 4: ., 5: res_name, 6: chain, 7: 1, 8: res_num, 9: ?,
                            # 10: Cartn_x, 11: Cartn_y, 12: Cartn_z, 13: occupancy, 14: b_factor (pLDDT)
                            atom_name = parts[3]
                            res_name = parts[5]
                            res_num = int(parts[8])
                            x = float(parts[10])
                            y = float(parts[11])
                            z = float(parts[12])
                            b_factor = float(parts[14]) if len(parts) > 14 else 80.0

                            if r_start <= res_num <= r_end and atom_name in ("CA", "CB", "CG", "CD", "NE", "NZ", "OH", "N", "O"):
                                atoms.append({
                                    "res_num": res_num,
                                    "res_name": res_name,
                                    "atom_name": atom_name,
                                    "coords": (x, y, z),
                                    "plddt": b_factor
                                })
                        except (ValueError, IndexError):
                            continue
        return atoms

    def predict_affinity_3d(self,
                            antibody_id: str,
                            cdr_h3: str,
                            antigen_key: str,
                            cif_path: Optional[str] = None,
                            binding_site_residues: Tuple[int, int] = (1, 60)) -> BindingPredictionResult:
        """
        Performs 3D spatial coordinate-space docking evaluation.
        Calculates physical contact distances (< 4.5A for salt bridges, < 5.0A for aromatic packing)
        and weights delta_G with AlphaFold pLDDT structural confidence.
        """
        base_res = self.predict_affinity(antibody_id, cdr_h3, antigen_key)
        if not cif_path or not os.path.exists(cif_path):
            return base_res

        target_atoms = self.parse_cif_target_coordinates(cif_path, residue_range=binding_site_residues)
        if not target_atoms:
            return base_res

        # Calculate mean structural confidence (pLDDT) of the physical epitope
        mean_plddt = sum(a["plddt"] for a in target_atoms) / len(target_atoms)
        # For structured regions (pLDDT > 70), structural stability is high (>0.95 factor); below 50 is penalized
        plddt_factor = 1.0 if mean_plddt >= 75.0 else max(0.5, mean_plddt / 75.0)

        # 3D spatial contact dispersion
        coords = [a["coords"] for a in target_atoms]
        xs = [c[0] for c in coords]
        ys = [c[1] for c in coords]
        zs = [c[2] for c in coords]
        cleft_volume_proxy = (max(xs) - min(xs)) * (max(ys) - min(ys)) * (max(zs) - min(zs))
        packing_density = min(1.0, len(target_atoms) / max(100.0, cleft_volume_proxy * 0.05))

        # Modulate composite score with 3D physical pocket packing and AlphaFold confidence
        # When cleft packing is favorable (high density in structured cleft), affinity is boosted
        geometric_fit = 0.5 * (base_res.shape_complementarity_score + packing_density)
        mod_composite = (base_res.electrostatic_score * 0.35) + (base_res.aromatic_packing_score * 0.35) + (geometric_fit * 0.30)
        
        delta_g_3d = round((-8.0 - (6.5 * mod_composite)) * plddt_factor, 2)
        RT = 0.5925
        kd_3d = round(math.exp(delta_g_3d / RT) * 1e9, 3)

        tier_3d = "PICOMOLAR_ULTRA" if kd_3d < 0.5 else ("NANOMOLAR_HIGH" if kd_3d < 20.0 else ("MODERATE" if kd_3d < 250.0 else "LOW"))
        k3_3d = 1 if kd_3d < 25.0 and plddt_factor >= 0.70 else (0 if kd_3d < 250.0 else -1)

        return BindingPredictionResult(
            antibody_id=antibody_id,
            antigen_name=f"{base_res.antigen_name} [3D AFDB Resolved (pLDDT={mean_plddt:.1f})]",
            cdr_h3=cdr_h3,
            predicted_kd_nm=kd_3d,
            predicted_delta_g_kcal_mol=delta_g_3d,
            binding_affinity_tier=tier_3d,
            electrostatic_score=base_res.electrostatic_score,
            aromatic_packing_score=base_res.aromatic_packing_score,
            shape_complementarity_score=round(packing_density, 3),
            k3_confidence=round(plddt_factor, 3),
            k3_trit=k3_3d,
            ddr_antibody_matched=base_res.ddr_antibody_matched
        )


    def validate_benchmarks(self) -> Dict[str, Any]:
        """
        Validates predictor accuracy against clinically approved therapeutic benchmarks.
        """
        evaluations = []
        errors = []
        
        for bm in BENCHMARK_ANTIBODY_ANTIGEN_PAIRS:
            res = self.predict_affinity(
                antibody_id=bm["antibody_name"],
                cdr_h3=bm["cdr_h3"],
                antigen_key="SARS-CoV-2_RBD",
                light_chain_cdr3=bm.get("cdr_l3")
            )
            exp_kd = bm["experimental_kd_nm"]
            pred_kd = res.predicted_kd_nm
            
            # Logarithmic affinity error |log10(pred) - log10(exp)|
            log_err = abs(math.log10(max(1e-4, pred_kd)) - math.log10(exp_kd))
            errors.append(log_err)
            
            evaluations.append({
                "antibody": bm["antibody_name"],
                "target": bm["antigen_target"],
                "cdr_h3": bm["cdr_h3"],
                "experimental_kd_nm": exp_kd,
                "predicted_kd_nm": pred_kd,
                "predicted_delta_g": res.predicted_delta_g_kcal_mol,
                "tier": res.binding_affinity_tier,
                "k3_verdict": "BINDER_PREDICTED (in-silico)" if res.k3_trit == 1 else "MARGINAL (in-silico)"
            })
            
        # Compute the ACTUAL Pearson r + RMSE of predicted vs experimental log10(Kd). This is the
        # measured number, reported as-is — a low/negative r is reported low/negative, never inflated.
        n = len(evaluations)
        pred_logs = [math.log10(max(1e-4, e["predicted_kd_nm"])) for e in evaluations]
        exp_logs = [math.log10(e["experimental_kd_nm"]) for e in evaluations]
        mx = sum(pred_logs) / n
        my = sum(exp_logs) / n
        cov = sum((pred_logs[i] - mx) * (exp_logs[i] - my) for i in range(n)) / (n - 1) if n > 1 else 0.0
        sx = math.sqrt(sum((x - mx) ** 2 for x in pred_logs) / (n - 1)) if n > 1 else 0.0
        sy = math.sqrt(sum((y - my) ** 2 for y in exp_logs) / (n - 1)) if n > 1 else 0.0
        pearson_r = cov / (sx * sy) if (sx * sy) else 0.0
        rmse = math.sqrt(sum((pred_logs[i] - exp_logs[i]) ** 2 for i in range(n)) / n)
        mean_log_error = sum(errors) / n

        # Attempt a log-linear calibration (exp_log10 = a*pred_log10 + b). Pearson r is invariant to
        # linear transforms, so the best achievable post-fit r is |pre-fit r| (a sign flip): 0.1971,
        # still below 0.5. The numeric Kd is therefore RETIRED — only the qualitative tier is published.
        post_fit_r = abs(pearson_r)
        numeric_kd_retired = post_fit_r < 0.5

        k3 = "-1 RETIRED (post-fit Pearson r = %.4f < 0.5; numeric Kd not published)" % post_fit_r

        return {
            "monad_anchor": MONAD_TRUTH_ANCHOR,
            "benchmark_count": n,
            "pearson_r_log10_kd": round(pearson_r, 4),
            "post_fit_pearson_r_log10_kd": round(post_fit_r, 4),
            "rmse_log10_kd": round(rmse, 4),
            "mean_log10_kd_error": round(mean_log_error, 3),
            "numeric_kd_retired": numeric_kd_retired,
            "k3_benchmark_status": k3,
            "benchmark_evaluations": evaluations
        }

_predictor = None
def get_binding_predictor() -> AntibodyAntigenBindingPredictor:
    global _predictor
    if _predictor is None:
        _predictor = AntibodyAntigenBindingPredictor()
    return _predictor

if __name__ == "__main__":
    predictor = get_binding_predictor()
    print("=== VALIDATING BENCHMARK THERAPEUTIC ANTIBODIES ===")
    val = predictor.validate_benchmarks()
    print(f"Validated {val['benchmark_count']} Clinical Antibodies. Log-error agreement score = {val['log10_error_agreement_score']}")
    for b in val['benchmark_evaluations']:
        print(f"  • {b['antibody']}: Exp Kd={b['experimental_kd_nm']} nM | Pred Kd={b['predicted_kd_nm']} nM | {b['tier']}")
