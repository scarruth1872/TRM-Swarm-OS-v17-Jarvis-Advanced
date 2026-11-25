#!/usr/bin/env python3
"""Verify case-based reasoning (task 573, council 10-4) on 406 real ChEMBL LpxC actives.

Leave-one-out inside the kernel: each case is predicted from the other 405, and the error is compared with a
mean predictor on the same folds. The acceptance test the council was given is enforced here: the CBR must beat
the mean predictor, or say that it did not.
"""
import json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=300) as r: return json.loads(r.read().decode())

def main():
    desc = json.loads((ROOT/"data"/"suite_lpxc_descriptors.json").read_text())
    feats, X, y = desc["features"], desc["X"], desc["y"]
    recs = json.loads((ROOT/"evidence"/"chembl_lpxc_actives.json").read_text())
    cases = []
    for i, (row, out) in enumerate(zip(X, y)):
        cid = recs[i].get("chembl_id") if i < len(recs) else f"case{i}"
        cases.append({"id": cid, "features": [float(v) for v in row], "outcome": float(out)})
    print(f"case base: {len(cases)} cases x {len(feats)} features ({', '.join(feats)})")

    for k in (3, 5, 9):
        r = post("/reason/case", {"cases": cases, "k": k, "mode": "loo"})
        print(f"  k={k}: RMSE {r['rmse']:.4f} | baseline {r['baseline_rmse']:.4f} | R² {r['r2']:.4f} "
              f"| worse_than_baseline {r['worse_than_baseline']}")

    best = None
    for k in (3, 5, 9):
        r = post("/reason/case", {"cases": cases, "k": k, "mode": "loo"})
        if best is None or r["rmse"] < best["rmse"]:
            best = r
    print(f"\nbest k={best['k']}: RMSE {best['rmse']:.4f} vs mean predictor {best['baseline_rmse']:.4f} "
          f"| R² {best['r2']:.4f}")

    # A query, to prove retrieval returns its working. The queried case is REMOVED from the base first:
    # querying a base that still contains the identical case returns distance 0.0 and the answer copied back,
    # which looks like a perfect prediction and is leakage, not reasoning.
    held = cases[0]
    q = {"cases": cases[1:], "k": 5, "mode": "query", "query": held["features"]}
    rq = post("/reason/case", q)
    print(f"\nquery for {held['id']} (removed from the base): predicted {rq['prediction']:.3f} "
          f"| actual {held['outcome']:.3f} | error {abs(rq['prediction']-held['outcome']):.3f}")
    print("  neighbours:", [(n["id"], round(n["distance"], 3), round(n["outcome"], 2)) for n in rq["neighbours"][:3]])

    ok = (not best["worse_than_baseline"]) and best["r2"] > 0.0
    print("\nRESULT:", "PASS (beats the mean predictor)" if ok else "FAIL (does not beat the mean predictor)")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
