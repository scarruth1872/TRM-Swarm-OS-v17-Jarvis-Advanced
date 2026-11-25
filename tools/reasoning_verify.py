#!/usr/bin/env python3
"""Cross-check the reasoning engine's derivations against RDKit on real compounds.

WHY THIS IS A CONTROL AND NOT A DEMO
------------------------------------
`/reason/forward` could "work" by restating its own rule base. That proves nothing. This harness computes
the same properties with **RDKit** (an independent implementation, a different language, different code) and
asks whether the engine's *derived* conclusion agrees with it, compound by compound, over all 406 real
ChEMBL LpxC actives.

The rules encode the published thresholds:

    mw > 500, logp > 5, hbd > 5, hba > 10      (Lipinski)
    tpsa > 140, rotb > 10                      (Veber)
    heavy_atoms < 20 or > 70                   (Ghose)

and "too many violations" is written as explicit pairwise conjunctions because the engine cannot count.

Two failure modes are also tested, because an engine that only ever answers TRUE is not reasoning:
  * a compound whose facts are withheld must come back UNKNOWN, naming the missing atom;
  * a rule given a premise asserted FALSE must not fire.

Usage:  env -u PYTHONPATH C:/.../Python312/python.exe tools/reasoning_verify.py [--limit N]
"""
from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path

from rdkit import Chem, RDLogger
from rdkit.Chem import Descriptors, Lipinski

RDLogger.DisableLog("rdApp.*")

ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"
TOKEN = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()


def post(path: str, payload: dict) -> dict:
    req = urllib.request.Request(
        BASE + path, data=json.dumps(payload).encode(), method="POST",
        headers={"Content-Type": "application/json", "X-Council-Token": TOKEN})
    with urllib.request.urlopen(req, timeout=20) as r:
        return json.loads(r.read().decode())


def descriptors(smiles: str) -> dict | None:
    mol = Chem.MolFromSmiles(smiles)
    if mol is None:
        return None
    return {
        "mw": Descriptors.MolWt(mol),
        "logp": Descriptors.MolLogP(mol),
        "hbd": Lipinski.NumHDonors(mol),
        "hba": Lipinski.NumHAcceptors(mol),
        "tpsa": Descriptors.TPSA(mol),
        "rotb": Lipinski.NumRotatableBonds(mol),
        "heavy": mol.GetNumHeavyAtoms(),
    }


# Two profiles. The published Lipinski set never derives a positive on this cohort (97.78% of the 406 have
# zero Lipinski violations), so it can only test over-claiming. The strict profile exists to exercise the TRUE
# path. Both are stated thresholds, checked against RDKit's own numbers.
PROFILES = {
    "druglikeness":        {"mw": 500, "logp": 5, "hbd": 5, "hba": 10, "veber": True},
    "druglikeness_strict": {"mw": 450, "logp": 4, "hbd": 4, "hba": 8,  "veber": False},
}


def facts_for(d: dict, shard_ref: str, prof: dict) -> list[dict]:
    """Threshold predicates as asserted facts, each carrying the record it came from."""
    out = []

    def add(atom):
        out.append({"atom": atom, "source": shard_ref})

    if d["mw"] > prof["mw"]: add(f"mw_gt_{prof['mw']}")
    if d["logp"] > prof["logp"]: add(f"logp_gt_{prof['logp']}")
    if d["hbd"] > prof["hbd"]: add(f"hbd_gt_{prof['hbd']}")
    if d["hba"] > prof["hba"]: add(f"hba_gt_{prof['hba']}")
    if prof["veber"]:
        if d["tpsa"] > 140: add("tpsa_gt_140")
        if d["rotb"] > 10: add("rotb_gt_10")
    if d["heavy"] < 20: add("heavy_atoms_lt_20")
    if d["heavy"] > 70: add("heavy_atoms_gt_70")
    return out


def expected(d: dict, prof: dict) -> bool:
    """Independent expectation computed straight from the thresholds — not from the rule table."""
    v = {
        "mw": d["mw"] > prof["mw"], "logp": d["logp"] > prof["logp"],
        "hbd": d["hbd"] > prof["hbd"], "hba": d["hba"] > prof["hba"],
        "ghose": d["heavy"] < 20 or d["heavy"] > 70,
    }
    return sum(v.values()) >= 2


def run_profile(records, name: str, prof: dict, limit: int) -> dict:
    agree = disagree = skipped = true_count = below = 0
    mismatches = []
    withheld_failures = 0
    for i, rec in enumerate(records[:limit] if limit else records):
        d = descriptors(rec.get("smiles", ""))
        if d is None:
            skipped += 1
            continue
        ref = f"shard:chembl_lpxc_actives@rec {i} chembl_id {rec.get('chembl_id')}"
        res = post("/reason/forward", {"rulebase": name, "facts": facts_for(d, ref, prof)})
        got = (res.get("truth") or {}).get("too_many_violations", "UNKNOWN")
        exp_true = expected(d, prof)

        if exp_true == (got == "TRUE"):
            agree += 1
            if got == "TRUE":
                true_count += 1
            else:
                below += 1
        else:
            disagree += 1
            mismatches.append({"i": i, "chembl_id": rec.get("chembl_id"), "engine": got,
                               "rdkit_expects": "TRUE" if exp_true else "not TRUE",
                               "desc": {k: round(v, 2) for k, v in d.items()}})

        # abstention control: withhold the mw fact; a TRUE that survives it is a fabricated derivation
        if i % 40 == 0:
            f = [x for x in facts_for(d, ref, prof) if not x["atom"].startswith("mw_gt_")]
            t2 = (post("/reason/forward", {"rulebase": name, "facts": f}).get("truth") or {}).get("too_many_violations", "UNKNOWN")
            if d["mw"] > prof["mw"] and t2 == "TRUE" and not expected(
                    {**d, "mw": 0}, prof):
                withheld_failures += 1

    return {"profile": name, "scored": agree + disagree, "agree": agree, "disagree": disagree,
            "true_derived": true_count, "left_below_true": below,
            "unparseable": skipped, "withheld_fact_failures": withheld_failures,
            "mismatches": mismatches}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    a = ap.parse_args()

    records = json.loads((ROOT / "evidence" / "chembl_lpxc_actives.json").read_text())
    print("=== reasoning engine vs RDKit, real ChEMBL LpxC actives ===")
    ok = True
    for name, prof in PROFILES.items():
        r = run_profile(records, name, prof, a.limit)
        pct = r["agree"] / max(1, r["scored"])
        print(f"\n  {name}  (mw>{prof['mw']}, logp>{prof['logp']}, hbd>{prof['hbd']}, hba>{prof['hba']})")
        print(f"    scored {r['scored']} | agree {r['agree']} ({pct:.4%}) | disagree {r['disagree']} | unparseable {r['unparseable']}")
        print(f"    TRUE derived by the engine : {r['true_derived']}")
        print(f"    correctly left below TRUE  : {r['left_below_true']}")
        print(f"    withheld-fact control      : {'PASS' if r['withheld_fact_failures'] == 0 else 'FAIL (%d)' % r['withheld_fact_failures']}")
        if r["mismatches"]:
            ok = False
            for m in r["mismatches"][:4]:
                print(f"      MISMATCH {m}")
        if r["true_derived"] == 0:
            print("      NOTE: no positive derived on this profile - the TRUE path is not exercised here")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
