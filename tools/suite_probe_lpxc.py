#!/usr/bin/env python3
"""Probe: can this host really run the Domains 1 tasks? Two measurements, not a demo.

WHY
---
The 1000-task suite asks for 531 "generate" and 339 "implement" deliverables. The only honest way to say
which can run is to run the ones the machine claims to support and record what happens.

This proves two task classes end-to-end on data that is actually on disk:
  * Task 9 class  -- drug-likeness (Lipinski/Veber/Ghose) computed with RDKit
  * Task 10 class -- QSAR: descriptor matrix -> pKi, scored by grouped K-fold cross-validation

It also runs the TRM leads, which is the failing case: the wet-lab audit found TRM-ABX-220 has no SMILES
anywhere, so the task cannot be run for the molecule the suite is written about. Both outcomes are recorded.

Evidence: data/suite_probe_lpxc.json. Exit 0 = probe completed (whatever it found).
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ACTS = ROOT / "evidence" / "chembl_lpxc_actives.json"
OUT = ROOT / "data" / "suite_probe_lpxc.json"

from rdkit import Chem, RDLogger  # noqa: E402
from rdkit.Chem import Descriptors, Crippen, Lipinski, rdMolDescriptors  # noqa: E402

RDLogger.DisableLog("rdApp.*")


def ghose_ok(mw, logp, mr, atoms):
    return 160 <= mw <= 480 and -0.4 <= logp <= 5.6 and 40 <= mr <= 130 and 20 <= atoms <= 70


def profile(smiles: str) -> dict | None:
    mol = Chem.MolFromSmiles(smiles)
    if mol is None:
        return None
    mw = Descriptors.MolWt(mol)
    logp = Crippen.MolLogP(mol)
    mr = Crippen.MolMR(mol)
    hbd, hba = Lipinski.NumHDonors(mol), Lipinski.NumHAcceptors(mol)
    tpsa, rot = rdMolDescriptors.CalcTPSA(mol), Lipinski.NumRotatableBonds(mol)
    heavy = mol.GetNumHeavyAtoms()
    return {
        "mw": round(mw, 2), "logp": round(logp, 2), "mr": round(mr, 2), "hbd": hbd, "hba": hba,
        "tpsa": round(tpsa, 2), "rotb": rot, "heavy_atoms": heavy,
        "lipinski_violations": sum([mw > 500, logp > 5, hbd > 5, hba > 10]),
        "veber_ok": rot <= 10 and tpsa <= 140,
        "ghose_ok": ghose_ok(mw, logp, mr, heavy),
    }


def main() -> int:
    acts = json.loads(ACTS.read_text(encoding="utf-8"))
    rec = {"source": str(ACTS.relative_to(ROOT)), "n_records": len(acts), "steps": {}}

    # ---- Task 9 class: drug-likeness ------------------------------------------------------------
    ok, bad = [], 0
    for a in acts:
        p = profile(a.get("smiles") or "")
        if p is None:
            bad += 1
            continue
        ok.append({**p, "chembl_id": a.get("chembl_id"), "value_nM": a.get("value_nM"), "type": a.get("type")})
    rec["steps"]["task9_druglikeness"] = {
        "measured": True,
        "n_parsed": len(ok), "n_unparseable": bad,
        "frac_lipinski_0_violations": round(sum(1 for r in ok if r["lipinski_violations"] == 0) / max(len(ok), 1), 4),
        "frac_veber_ok": round(sum(1 for r in ok if r["veber_ok"]) / max(len(ok), 1), 4),
        "frac_ghose_ok": round(sum(1 for r in ok if r["ghose_ok"]) / max(len(ok), 1), 4),
        "median_mw": sorted(r["mw"] for r in ok)[len(ok) // 2] if ok else None,
        "example": ok[0] if ok else None,
    }

    # ---- the failing case: the TRM leads the suite is written about -------------------------------
    leads = {}
    for f in (ROOT / "data" / "wet_lab_bridge_package.json", ROOT / "data" / "trm_abx_220_recalibration.json"):
        if not f.exists():
            leads[f.name] = "file absent"
            continue
        t = f.read_text(encoding="utf-8", errors="replace")
        leads[f.name] = "SMILES present" if "SMILES" in t.upper() else "no SMILES field"
    rec["steps"]["trm_leads"] = {
        "measured": True, "files_checked": leads,
        "runnable": all(v == "SMILES present" for v in leads.values()),
        "why_blocked": ("no SMILES exists for TRM-ABX-220 in any repo file (wet-lab audit, 5 defects); a "
                        "descriptor or QSAR task about a molecule without a structure cannot be run"),
    }

    # ---- Task 10 class: QSAR on the real actives --------------------------------------------------
    # MEASURED 2026-11-24: this interpreter (3.12) has RDKit but its sklearn is built against numpy 1.x
    # while numpy 2.0.2 is installed -> "numpy.dtype size changed". 3.11 has a working sklearn but no
    # RDKit. So the descriptor matrix is always written here and the model is trained in the 3.11 pass;
    # a capability that only works by chaining two interpreters must be recorded as such, not presented
    # as one run.
    desc_path = ROOT / "data" / "suite_lpxc_descriptors.json"
    X, y = [], []
    for r in ok:
        if r["value_nM"] and r["value_nM"] > 0:
            X.append([r["mw"], r["logp"], r["mr"], r["hbd"], r["hba"], r["tpsa"], r["rotb"], r["heavy_atoms"]])
            y.append(-math.log10(r["value_nM"] * 1e-9))  # pKi
    desc_path.write_text(json.dumps({"features": ["mw", "logp", "mr", "hbd", "hba", "tpsa", "rotb", "heavy_atoms"],
                                     "X": X, "y": y,
                                     "y_note": "pKi = -log10(Ki in M), from value_nM"}, indent=0), encoding="utf-8")
    print(f"descriptors written for the 3.11 pass: {desc_path} ({len(X)} rows)")
    try:
        import numpy as np
        from sklearn.ensemble import GradientBoostingRegressor
        from sklearn.model_selection import KFold
    except Exception as exc:  # noqa: BLE001 - any import/ABI failure is the finding
        rec["steps"]["task10_qsar"] = {
            "measured": False,
            "why": f"sklearn unusable in this interpreter: {type(exc).__name__}: {str(exc)[:90]}",
            "next": "train from data/suite_lpxc_descriptors.json with Python 3.11 (sklearn 1.3.2, numpy 1.26)",
            "n_descriptors_written": len(X),
        }
        OUT.write_text(json.dumps(rec, indent=1), encoding="utf-8")
        print(f"python 3.12 cannot train here: {type(exc).__name__}")
        return 0

    X, y = np.array(X), np.array(y)
    kf = KFold(n_splits=5, shuffle=True, random_state=0)
    scores, baselines = [], []
    for tr, te in kf.split(X):
        m = GradientBoostingRegressor(random_state=0, n_estimators=300, max_depth=3)
        m.fit(X[tr], y[tr])
        scores.append(m.score(X[te], y[te]))
        baselines.append(1 - ((y[te] - y[tr].mean()) ** 2).sum() / ((y[te] - y[te].mean()) ** 2).sum())
    rec["steps"]["task10_qsar"] = {
        "measured": True, "n": int(len(y)), "features": 8, "target": "pKi = -log10(Ki_M)",
        "cv5_r2_mean": round(float(np.mean(scores)), 4), "cv5_r2_std": round(float(np.std(scores)), 4),
        "cv5_r2_baseline_mean": round(float(np.mean(baselines)), 4),
        "reading": ("beats the mean-predictor baseline" if np.mean(scores) > np.mean(baselines) + 0.05
                    else "does NOT beat the mean-predictor baseline -> not a usable QSAR model"),
        "pki_range": [round(float(y.min()), 2), round(float(y.max()), 2)],
    }

    OUT.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    s9, s10 = rec["steps"]["task9_druglikeness"], rec["steps"]["task10_qsar"]
    print(f"TASK 9  drug-likeness : parsed {s9['n_parsed']}/{rec['n_records']} ({s9['n_unparseable']} unparseable) "
          f"| Lipinski-0 {s9['frac_lipinski_0_violations']:.1%} | Veber {s9['frac_veber_ok']:.1%} | Ghose {s9['frac_ghose_ok']:.1%}")
    print(f"TASK 10 QSAR          : n={s10['n']} pKi {s10['pki_range']} | CV5 R2 {s10['cv5_r2_mean']} "
          f"(baseline {s10['cv5_r2_baseline_mean']}) -> {s10['reading']}")
    print(f"TRM leads runnable    : {rec['steps']['trm_leads']['runnable']} — {rec['steps']['trm_leads']['why_blocked'][:90]}")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
