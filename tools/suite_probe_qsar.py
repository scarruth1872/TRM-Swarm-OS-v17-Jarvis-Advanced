#!/usr/bin/env python3
"""QSAR training pass (Python 3.11: sklearn 1.3.2 + numpy 1.26).

Reads the descriptor matrix produced by tools/suite_probe_lpxc.py (Python 3.12 + RDKit) and scores a model
by 5-fold cross-validation against a mean-predictor baseline. A model that cannot beat the baseline is
reported as unusable, not as a result.

Usage: python tools/suite_probe_qsar.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data" / "suite_lpxc_descriptors.json"
LPXC = ROOT / "data" / "suite_probe_lpxc.json"


def main() -> int:
    if not SRC.exists():
        print(f"QSAR: {SRC} missing — run tools/suite_probe_lpxc.py with Python 3.12 first")
        return 2
    d = json.loads(SRC.read_text(encoding="utf-8"))

    import numpy as np
    from sklearn.ensemble import GradientBoostingRegressor
    from sklearn.model_selection import KFold

    X, y = np.array(d["X"], dtype=float), np.array(d["y"], dtype=float)
    kf = KFold(n_splits=5, shuffle=True, random_state=0)
    scores, baselines = [], []
    for tr, te in kf.split(X):
        if len(te) == 0:
            continue
        m = GradientBoostingRegressor(random_state=0, n_estimators=300, max_depth=3)
        m.fit(X[tr], y[tr])
        scores.append(m.score(X[te], y[te]))
        baselines.append(1 - ((y[te] - y[tr].mean()) ** 2).sum() / ((y[te] - y[te].mean()) ** 2).sum())

    mean_r2, std_r2, base = float(np.mean(scores)), float(np.std(scores)), float(np.mean(baselines))
    res = {
        "measured": True, "interpreter": sys.version.split()[0], "n": int(len(y)),
        "features": d["features"], "target": d["y_note"],
        "cv5_r2_mean": round(mean_r2, 4), "cv5_r2_std": round(std_r2, 4),
        "cv5_r2_baseline_mean": round(base, 4),
        "pki_range": [round(float(y.min()), 2), round(float(y.max()), 2)],
        "reading": ("beats the mean-predictor baseline" if mean_r2 > base + 0.05
                    else "does NOT beat the mean-predictor baseline -> not a usable QSAR model"),
    }
    if LPXC.exists():
        rec = json.loads(LPXC.read_text(encoding="utf-8"))
        rec["steps"]["task10_qsar"] = res
        LPXC.write_text(json.dumps(rec, indent=1), encoding="utf-8")

    print(f"QSAR (py{sys.version_info.major}.{sys.version_info.minor}) n={res['n']} pKi {res['pki_range']} | "
          f"CV5 R2 {mean_r2:.4f} ± {std_r2:.4f} | baseline {base:.4f} -> {res['reading']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
