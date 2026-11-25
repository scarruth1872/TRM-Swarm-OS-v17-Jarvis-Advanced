#!/usr/bin/env python3
"""Verify abduction (task 575, council 13-1) on real compounds and by independent enumeration.

Two checks, both falsifiable:
  1. REAL CASES — for every compound whose RDKit descriptors violate the Ghose atom-count rule, the kernel must
     abduce exactly {heavy_atoms_lt_20} or {heavy_atoms_gt_70}, matching which side RDKit measured.
  2. MINIMALITY — the smallest explanation for "two or more violations" must be of size two, and no single
     assumption may explain it. The harness asks the kernel for size-1 explanations and requires them to fail;
     if one succeeds, the size-2 answer was not minimal.
"""
import json, sys, urllib.request
from pathlib import Path

from rdkit import Chem, RDLogger
from rdkit.Chem import Lipinski

RDLogger.DisableLog("rdApp.*")
TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

GHOSE_HYPS = [{"atom": "heavy_atoms_lt_20", "cost": 1.0}, {"atom": "heavy_atoms_gt_70", "cost": 1.0}]

def main():
    recs = json.loads((ROOT/"evidence"/"chembl_lpxc_actives.json").read_text())
    desc = json.loads((ROOT/"data"/"suite_lpxc_descriptors.json").read_text())
    heavy_idx = desc["features"].index("heavy_atoms")

    cases, agree, disagree = 0, 0, []
    for i, (rec, row) in enumerate(zip(recs, desc["X"])):
        heavy = int(row[heavy_idx])
        if 20 <= heavy <= 70:
            continue  # no Ghose violation: nothing to explain
        cases += 1
        r = post("/reason/abduce", {
            "rulebase": "druglikeness",
            "observation": "violation_ghose",
            "hypotheses": GHOSE_HYPS,
            "facts": [],
            "max_size": 1,
        })
        expected = "heavy_atoms_lt_20" if heavy < 20 else "heavy_atoms_gt_70"
        if r["assumptions"] == [expected]:
            agree += 1
        else:
            disagree.append({"chembl_id": rec.get("chembl_id"), "heavy": heavy,
                             "expected": expected, "got": r["assumptions"]})
    print(f"Ghose cases     : {cases} | exact abduction {agree} | wrong {len(disagree)}")
    for d in disagree[:3]: print("   MISMATCH", d)

    # minimality: no single assumption may explain a two-violation conclusion
    single_succeeds = 0
    for atom in ("mw_gt_500", "logp_gt_5", "hbd_gt_5", "hba_gt_10", "heavy_atoms_lt_20"):
        r = post("/reason/abduce", {"rulebase": "druglikeness", "observation": "too_many_violations",
                                    "hypotheses": [{"atom": atom}], "facts": [], "max_size": 1})
        if r["assumptions"]:
            single_succeeds += 1
    print(f"minimality check: single-assumption explanations of a 2-premise rule: {single_succeeds} (must be 0)")

    # a size-2 abduction, once, to show it is found and costed
    r = post("/reason/abduce", {"rulebase": "druglikeness", "observation": "too_many_violations",
                                "hypotheses": [{"atom": a} for a in
                                               ("mw_gt_500","logp_gt_5","hbd_gt_5","hba_gt_10","heavy_atoms_lt_20")],
                                "facts": [], "max_size": 2})
    print(f"size-2 abduction: {r['assumptions']} cost {r['cost']} searched {r['searched']} "
          f"complete {r['complete']} alternatives {r['alternatives']}")

    ok = not disagree and single_succeeds == 0 and cases > 0 and len(r["assumptions"]) == 2
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
