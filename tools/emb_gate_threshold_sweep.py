"""Threshold sweep for the embedding gate - is the OPERATING POINT the problem?"""
import json
import os
import pickle
import sys

import numpy as np

os.chdir(r"F:/Development sites/TRM-Swarm-OS-v2")
sys.path.insert(0, r"F:/Development sites/TRM-Swarm-OS-v2/tools")
import build_embedding_gate as g  # noqa: E402

clf = pickle.load(open(g.MODEL_OUT, "rb"))
hold_t, hold_w = g.load_holdout()
s3 = g.load_set3()
s3_t = [t for t, _, _ in s3]
s3_w = [w for _, w, _ in s3]

Xh = g.embed(hold_t)
X3 = g.embed(s3_t)


def sweep(X, y, name):
    p = clf.predict_proba(X)[:, 0]
    y = np.asarray(y)
    danger = y == 0
    benign = ~danger
    print(f"\n{name}  (n={len(y)}, danger={int(danger.sum())})")
    print("  tau  danger-recall  danger-PROCEED  benign-false-block")
    for tau in (0.5, 0.4, 0.3, 0.2, 0.1):
        blocked = p >= tau
        rec = float((blocked & danger).sum() / max(1, danger.sum()))
        pt = float((~blocked & danger).sum() / max(1, danger.sum()))
        fb = float((blocked & benign).sum() / max(1, benign.sum()))
        print(f"  {tau:3.1f}   {rec:11.2f}    {pt:12.2f}    {fb:16.2f}")
    return p


sweep(Xh, hold_w, "fresh holdout (24, written after the corpus froze)")
sweep(X3, s3_w, "SET3 (20 danger / 60 benign) - contaminated")

p_h = clf.predict_proba(Xh)[:, 0]
y = np.asarray(hold_w)
print("\nholdout DANGER items, P(BLOCK) ascending (the pass-through risk):")
for pr, t in sorted(zip(p_h[y == 0], [t for t, w, _ in g.load_holdout() if w == 0])):
    print(f"   {pr:.3f}  {t[:74]}")
print("\nholdout BENIGN items that would block at tau=0.2:")
for pr, t in sorted(zip(p_h[y != 0], [t for t, w, _ in g.load_holdout() if w != 0]), reverse=True):
    if pr >= 0.2:
        print(f"   {pr:.3f}  {t[:74]}")
