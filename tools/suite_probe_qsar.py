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
    # ---- predict the three leads, with an applicability-domain check -------------------------------
    # A prediction from a model trained on 406 actives says nothing useful about a molecule outside that
    # model's range. So each lead gets its descriptor distance from the training set, and the extrapolation
    # is stated rather than hidden.
    leads_path = ROOT / "data" / "suite_leads_descriptors.json"
    if leads_path.exists():
        leads = json.loads(leads_path.read_text(encoding="utf-8"))["leads"]
        full = GradientBoostingRegressor(random_state=0, n_estimators=300, max_depth=3).fit(X, y)
        mu, sd = X.mean(axis=0), X.std(axis=0)
        preds = {}
        for name, e in leads.items():
            feats = e.get("model_features")
            if not feats:
                preds[name] = {"error": e.get("error", "no features")}
                continue
            f = np.array([feats], dtype=float)
            z = np.abs((f[0] - mu) / np.where(sd == 0, 1, sd))
            outs = [t.predict(f)[0] for t in (GradientBoostingRegressor(random_state=0, n_estimators=300,
                                                                       max_depth=3).fit(X[tr], y[tr])
                                              for tr, _ in KFold(n_splits=5, shuffle=True,
                                                                 random_state=0).split(X))]
            preds[name] = {
                "predicted_pKi": round(float(full.predict(f)[0]), 2),
                "fold_prediction_spread": [round(float(min(outs)), 2), round(float(max(outs)), 2)],
                "max_abs_z": round(float(z.max()), 2),
                "outside_training_range_features": [res["features"][i] for i in range(len(z)) if z[i] > 2],
                "in_domain": bool(z.max() <= 2),
                "training_mean_max_z": round(float(np.abs((X - mu) / np.where(sd == 0, 1, sd)).max()), 2),
            }
        res["lead_predictions"] = preds
        print("\nlead predictions (pKi from the 406-actives model):")
        for name, pr in preds.items():
            if "error" in pr:
                print(f"  {name}: {pr['error']}"); continue
            flag = "in domain" if pr["in_domain"] else f"EXTRAPOLATION (max |z| {pr['max_abs_z']}, outside: {pr['outside_training_range_features']})"
            print(f"  {name}: pKi {pr['predicted_pKi']} (fold spread {pr['fold_prediction_spread']}) -> {flag}")

    if LPXC.exists():
        rec = json.loads(LPXC.read_text(encoding="utf-8"))
        rec["steps"]["task10_qsar"] = res
        LPXC.write_text(json.dumps(rec, indent=1), encoding="utf-8")

    print(f"QSAR (py{sys.version_info.major}.{sys.version_info.minor}) n={res['n']} pKi {res['pki_range']} | "
          f"CV5 R2 {mean_r2:.4f} ± {std_r2:.4f} | baseline {base:.4f} -> {res['reading']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
