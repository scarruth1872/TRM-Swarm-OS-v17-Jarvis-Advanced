#!/usr/bin/env python3
"""Verify analogical reasoning (task 574, council 12-2) on 406 real compounds, with a control.

Acceptance, pre-registered in the proposal: the analogy must beat **zero-delta** (copying the neighbour's
outcome) on leave-one-out, and the **random-outcome control** must show no such gain. If the control gains as
much as the real run, the mechanism is fitting noise.
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
    recs = json.loads((ROOT/"evidence"/"chembl_lpxc_actives.json").read_text())
    cases = [{"id": (recs[i].get("chembl_id") if i < len(recs) else f"c{i}"),
              "features": [float(v) for v in row], "outcome": float(y)}
             for i, (row, y) in enumerate(zip(desc["X"], desc["y"]))]
    print(f"case base: {len(cases)} compounds x {len(desc['features'])} descriptors")

    r = post("/reason/analogy", {"cases": cases, "mode": "loo"})
    print(f"leave-one-out : analogy RMSE {r['analogy_rmse']:.4f} | zero-delta {r['zero_delta_rmse']:.4f} "
          f"| mean {r['mean_rmse']:.4f} | R² {r['r2']:.4f}")
    print(f"                clamped (beyond observed transformation): {r['clamped_fraction']:.1%}")
    print(f"                worse_than_zero_delta: {r['worse_than_zero_delta']}")

    c = post("/reason/analogy", {"cases": cases, "mode": "control", "seed": 42})
    gain_real = (r["zero_delta_rmse"] - r["analogy_rmse"]) / r["zero_delta_rmse"]
    gain_ctrl = (c["zero_delta_rmse"] - c["analogy_rmse"]) / c["zero_delta_rmse"]
    print(f"control (seed 42): analogy {c['analogy_rmse']:.4f} | zero-delta {c['zero_delta_rmse']:.4f}")
    print(f"relative gain : real {gain_real:+.2%} | control {gain_ctrl:+.2%}")

    ok = (not r["worse_than_zero_delta"]) and gain_real > 0 and gain_ctrl <= 0.10
    print("RESULT:", "PASS" if ok else "FAIL")
    if not ok:
        print("  acceptance requires: analogy beats zero-delta, and the control gains at most 10%")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
