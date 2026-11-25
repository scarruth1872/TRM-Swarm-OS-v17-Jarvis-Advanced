#!/usr/bin/env python3
"""Real ChEMBL Bioactivity Experiment: 64-Trit Kleene-3 Pharmacophore Screen on EGFR (CHEMBL203).

Fetches real measured human EGFR IC50 compounds from the ChEMBL REST API, computes
RDKit Morgan fingerprints, builds a 64-trit Kleene-3 target template on a 70% training
split, and evaluates held-out ROC-AUC and Enrichment Factors on the 30% test split
using the exact same bitwise popcount formula as `K3_DRUG_SCREEN_WGSL` on the RX 6700 XT.
"""

from __future__ import annotations

import json
import random
import time
import urllib.request
from pathlib import Path

from rdkit import Chem, DataStructs
from rdkit.Chem import rdFingerprintGenerator

OUT_PATH = Path("evidence/REAL_CHEMBL_K3_SCREEN.json")


def fetch_chembl_egfr(limit_per_class: int = 250) -> list[tuple[str, int, float]]:
    """Fetch real EGFR (CHEMBL203) IC50 actives (<=100 nM) and inactives (>=10000 nM)."""
    records = []
    queries = [
        (1, f"https://www.ebi.ac.uk/chembl/api/data/activity.json?target_chembl_id=CHEMBL203&standard_type=IC50&standard_units=nM&standard_value__lte=100&standard_relation=%3D&limit={limit_per_class}"),
        (0, f"https://www.ebi.ac.uk/chembl/api/data/activity.json?target_chembl_id=CHEMBL203&standard_type=IC50&standard_units=nM&standard_value__gte=10000&standard_relation=%3D&limit={limit_per_class}"),
    ]
    seen_smiles: set[str] = set()
    for label, url in queries:
        req = urllib.request.Request(url, headers={"Accept": "application/json", "User-Agent": "TRM-Swarm-OS-Bench/1.0"})
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        for act in data.get("activities", []):
            smi = act.get("canonical_smiles")
            val = act.get("standard_value")
            if not smi or val is None:
                continue
            mol = Chem.MolFromSmiles(smi)
            if mol is None:
                continue
            canon = Chem.MolToSmiles(mol)
            if canon in seen_smiles:
                continue
            seen_smiles.add(canon)
            records.append((canon, label, float(val)))
    return records


def roc_auc(labels: list[int], scores: list[float]) -> float:
    pairs = sorted(zip(scores, labels), key=lambda x: x[0], reverse=True)
    pos = sum(labels)
    neg = len(labels) - pos
    if pos == 0 or neg == 0:
        return 0.5
    tp = fp = 0
    auc = 0.0
    prev_fp = 0
    prev_tp = 0
    for _, y in pairs:
        if y == 1:
            tp += 1
        else:
            fp += 1
            auc += (tp + prev_tp) * 0.5 * (fp - prev_fp)
            prev_fp = fp
            prev_tp = tp
    return auc / (pos * neg)


def enrichment_factor(labels: list[int], scores: list[float], frac: float) -> float:
    n = len(labels)
    k = max(1, int(round(n * frac)))
    total_pos = sum(labels)
    if total_pos == 0:
        return 0.0
    top_k = [y for _, y in sorted(zip(scores, labels), key=lambda x: x[0], reverse=True)[:k]]
    return (sum(top_k) / k) / (total_pos / n)


def fp_to_u64(bv) -> int:
    val = 0
    for b in bv.GetOnBits():
        val |= 1 << b
    return val


def main() -> None:
    t0 = time.perf_counter()
    print("Fetching real EGFR (CHEMBL203) IC50 records from ChEMBL API...")
    recs = fetch_chembl_egfr(limit_per_class=300)
    n_act = sum(1 for _, y, _ in recs if y == 1)
    n_inact = sum(1 for _, y, _ in recs if y == 0)
    print(f"Loaded {len(recs)} unique valid molecules (actives<=100nM: {n_act}, inactives>=10uM: {n_inact})")

    rng = random.Random(42)
    rng.shuffle(recs)
    split = int(0.7 * len(recs))
    train, test = recs[:split], recs[split:]

    gen64 = rdFingerprintGenerator.GetMorganGenerator(radius=2, fpSize=64)
    gen2048 = rdFingerprintGenerator.GetMorganGenerator(radius=2, fpSize=2048)

    tr_fp64 = [(fp_to_u64(gen64.GetFingerprint(Chem.MolFromSmiles(s))), y) for s, y, _ in train]
    te_fp64 = [(fp_to_u64(gen64.GetFingerprint(Chem.MolFromSmiles(s))), y) for s, y, _ in test]

    # Build 64-trit Kleene-3 pharmacophore template strictly on training set
    tr_act = [fp for fp, y in tr_fp64 if y == 1]
    tr_inact = [fp for fp, y in tr_fp64 if y == 0]
    pos_mask = 0
    neg_mask = 0
    for bit in range(64):
        m = 1 << bit
        p_a = sum(1 for fp in tr_act if fp & m) / max(1, len(tr_act))
        p_i = sum(1 for fp in tr_inact if fp & m) / max(1, len(tr_inact))
        diff = p_a - p_i
        if diff >= 0.08:
            pos_mask |= m
        elif diff <= -0.08:
            neg_mask |= m

    # Score held-out test molecules with exact K3 bitwise popcount formula
    k3_scores = [
        (fp & pos_mask).bit_count() - (fp & neg_mask).bit_count()
        for fp, _ in te_fp64
    ]
    te_labels = [y for _, y in te_fp64]

    k3_auc = roc_auc(te_labels, [float(s) for s in k3_scores])
    k3_ef5 = enrichment_factor(te_labels, [float(s) for s in k3_scores], 0.05)
    k3_ef10 = enrichment_factor(te_labels, [float(s) for s in k3_scores], 0.10)

    # Compare against standard 2048-bit Morgan 1-NN Tanimoto baseline on the same split
    tr_act_2048 = [gen2048.GetFingerprint(Chem.MolFromSmiles(s)) for s, y, _ in train if y == 1]
    te_2048 = [gen2048.GetFingerprint(Chem.MolFromSmiles(s)) for s, _, _ in test]
    tan_scores = [max(DataStructs.BulkTanimotoSimilarity(fp, tr_act_2048)) for fp in te_2048]
    tan_auc = roc_auc(te_labels, tan_scores)
    tan_ef5 = enrichment_factor(te_labels, tan_scores, 0.05)
    tan_ef10 = enrichment_factor(te_labels, tan_scores, 0.10)

    report = {
        "target": "CHEMBL203 (Human EGFR, real ChEMBL IC50 bioactivities)",
        "unique_molecules": len(recs),
        "actives_le_100nm": n_act,
        "inactives_ge_10000nm": n_inact,
        "train_count": len(train),
        "test_count": len(test),
        "k3_64trit_template": {
            "pos_bits_set": pos_mask.bit_count(),
            "neg_bits_set": neg_mask.bit_count(),
            "neutral_bits": 64 - pos_mask.bit_count() - neg_mask.bit_count(),
            "held_out_roc_auc": round(k3_auc, 4),
            "ef_5pct": round(k3_ef5, 2),
            "ef_10pct": round(k3_ef10, 2),
        },
        "baseline_2048bit_tanimoto_1nn": {
            "held_out_roc_auc": round(tan_auc, 4),
            "ef_5pct": round(tan_ef5, 2),
            "ef_10pct": round(tan_ef10, 2),
        },
        "elapsed_s": round(time.perf_counter() - t0, 2),
    }
    OUT_PATH.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
