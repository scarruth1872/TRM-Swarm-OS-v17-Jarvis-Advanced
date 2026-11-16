#!/usr/bin/env python3
"""Train a SPARSE-GATING ROUTER on labels this harness already produces.

The motivation is measured, not theoretical: the council's 15 CPU ternary heads score AUC 0.493 against "will the
small model hallucinate", so gating on them routes 60 % of work to a tier 13x slower for zero quality gain
(docs/PHASE-6-GAPS.md G118). This trains a router on the actual outcome labels instead - `schema_ok` and
strict grounding from tools/loadtest_tasks.py - and refuses to report anything it did not measure on held-out FILES.

Design choices that matter for honesty:
  * Split by FILE, not by row. The same source file yields up to 5 tasks (one per shape); a row split would put the
    same excerpt on both sides and inflate the score.
  * Features are cheap, deterministic and CPU-only (no model, no GPU): excerpt statistics plus the k3 head's
    ternary probabilities from the live council. That is what makes routing affordable: ~0.7 ms/head call.
  * The reported number is AUC against the *hard* label (the 1.5B failed schema OR strict grounding), plus the
    coverage/quality curve that a deployment would actually choose from.

Usage:
  python tools/train_router.py [--labels scratch/loadtest/run1000_clean/results.jsonl] [--out scratch/router]
"""
from __future__ import annotations

import argparse
import json
import math
import random
import re
import sys
import urllib.request
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
ID_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_\.:]{2,}$")
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def strict_ok(rec: dict) -> bool:
    return bool(rec.get("grounded")) and all(ID_RE.match(s) for s in (rec.get("symbols") or []))


def k3_features(text: str, heads: list[str], base: str = "http://127.0.0.1:8022") -> list[float]:
    """Ternary-head probabilities from the live council. Returns zeros if the council is down, and says so."""
    pa, pb, agree = [], [], []
    for h in heads:
        try:
            req = urllib.request.Request(f"{base}/k3/classify",
                                         data=json.dumps({"agent": h, "text": text[:1200]}).encode(),
                                         headers={"Content-Type": "application/json"})
            d = json.loads(urllib.request.urlopen(req, timeout=10).read())
            p = d["probs"]
            pa.append(p["analyze"]); pb.append(p["block"]); agree.append(d["label"])
        except Exception:
            return [0.0, 0.0, 0.0]
    return [float(np.mean(pa)), float(np.mean(pb)),
            max(Counter(agree).values()) / max(len(agree), 1)]


def text_features(excerpt: str, task: str, path: str) -> list[float]:
    lines = excerpt.splitlines()
    n = max(len(excerpt), 1)
    blank = sum(1 for l in lines if not l.strip())
    comments = sum(1 for l in lines if l.strip().startswith(("#", "//", "/*", "*")))
    indents = [len(l) - len(l.lstrip()) for l in lines if l.strip()]
    tokens = TOKEN_RE.findall(excerpt)
    low = excerpt.lower()
    counts = {k: low.count(k) for k in ("def ", "fn ", "class ", "impl ", "async ", "await ", "try",
                                        "except", "return", "import ", "pub ", "match ", "unwrap")}
    shapes = ["testability", "docstring", "failure", "risk", "coupling"]
    ent = 0.0
    for _, c in Counter(excerpt).most_common():
        p = c / n
        ent -= p * math.log2(p)
    lang = [1.0 if path.endswith(".rs") else 0.0, 1.0 if path.endswith(".py") else 0.0,
            1.0 if path.endswith(".ts") else 0.0]
    return [
        len(excerpt), len(lines), blank / max(len(lines), 1), comments / max(len(lines), 1),
        float(np.mean(indents)) if indents else 0.0, float(np.max(indents)) if indents else 0.0,
        len(tokens), len(set(tokens)), len(set(tokens)) / max(len(tokens), 1),
        float(np.mean([len(t) for t in tokens])) if tokens else 0.0,
        ent, max((len(l) for l in lines), default=0),
        *counts.values(), *lang, *[1.0 if task == s else 0.0 for s in shapes],
    ]


def auc(scores: np.ndarray, labels: np.ndarray) -> float:
    """Rank-based AUC; labels True = hard (positive)."""
    pos = scores[labels]
    neg = scores[~labels]
    if len(pos) == 0 or len(neg) == 0:
        return float("nan")
    order = np.argsort(np.concatenate([pos, neg]))
    ranks = np.empty(len(order), float)
    ranks[order] = np.arange(1, len(order) + 1)
    rp = ranks[:len(pos)].sum()
    return float((rp - len(pos) * (len(pos) + 1) / 2) / (len(pos) * len(neg)))


def train_logreg(X: np.ndarray, y: np.ndarray, epochs: int = 800, lr: float = 0.08) -> tuple[np.ndarray, float]:
    mu, sd = X.mean(0), X.std(0) + 1e-6
    Z = (X - mu) / sd
    w = np.zeros(Z.shape[1])
    b = 0.0
    for _ in range(epochs):
        p = 1 / (1 + np.exp(-(Z @ w + b)))
        g = p - y
        w -= lr * (Z.T @ g / len(y) + 1e-3 * w)
        b -= lr * g.mean()
    return np.concatenate([[b], w]), mu, sd


def predict(params: np.ndarray, X: np.ndarray, mu, sd) -> np.ndarray:
    Z = (X - mu) / (sd + 1e-6)
    return 1 / (1 + np.exp(-(Z @ params[1:] + params[0])))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--labels", default="scratch/loadtest/run1000_clean/results.jsonl")
    ap.add_argument("--out", default="scratch/router")
    ap.add_argument("--heads", type=int, default=4, help="k3 heads to query (each ~0.7 ms)")
    args = ap.parse_args()

    outdir = ROOT / args.out
    outdir.mkdir(parents=True, exist_ok=True)
    recs = [json.loads(l) for l in (ROOT / args.labels).read_text(encoding="utf-8").splitlines() if l.strip()]
    recs = [r for r in recs if r["ok"]]
    try:
        heads = [h["agent"] for h in json.loads(
            urllib.request.urlopen("http://127.0.0.1:8022/k3/heads", timeout=10).read())["heads"]][: args.heads]
        k3_live = True
    except Exception:
        heads, k3_live = [], False
    print(f"[data] {len(recs)} labelled tasks | k3 heads live={k3_live} ({len(heads)})")

    X, y, files, shapes = [], [], [], []
    for r in recs:
        ex = (ROOT / r["file"]).read_text(encoding="utf-8", errors="replace")[:2400]
        feats = text_features(ex, r["task"], r["file"])
        if k3_live:
            feats += k3_features(f"{r['task']} {r['file']}\n{ex}", heads)
        X.append(feats); y.append(0.0 if (r.get("schema_ok") and strict_ok(r)) else 1.0)
        files.append(r["file"]); shapes.append(r["task"])
    X = np.array(X, float); y = np.array(y, float)

    # ---- split by FILE -------------------------------------------------------
    uniq = sorted(set(files))
    rng = random.Random(17)
    rng.shuffle(uniq)
    cut = int(0.75 * len(uniq))
    train_files = set(uniq[:cut])
    tr = np.array([f in train_files for f in files])
    print(f"[split] {len(train_files)} train files / {len(uniq)-len(train_files)} held-out files "
          f"| rows {tr.sum()} / {(~tr).sum()} | hard rate {y.mean():.3f}")

    params, mu, sd = train_logreg(X[tr], y[tr])
    p_tr = predict(params, X[tr], mu, sd)
    p_te = predict(params, X[~tr], mu, sd)
    res = {
        "protocol": "SPARSE_GATE_ROUTER_v1",
        "labels_source": args.labels,
        "k3_heads_live": k3_live, "k3_heads": heads,
        "n_rows": int(len(y)), "n_train_files": len(train_files), "n_heldout_files": len(uniq) - cut,
        "hard_rate": round(float(y.mean()), 4),
        "auc_train": round(auc(p_tr, y[tr] == 1), 4),
        "auc_heldout": round(auc(p_te, y[~tr] == 1), 4),
        "baseline_k3_auc": 0.493,
    }

    # ---- coverage / quality curve on held-out rows ---------------------------
    curve = []
    for t in (0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9):
        route_up = p_te >= t
        hard_te = (y[~tr] == 1)
        curve.append({
            "threshold": t,
            "routed_to_big_pct": round(100 * route_up.mean(), 1),
            "hard_caught_pct": round(100 * (route_up & hard_te).sum() / max(hard_te.sum(), 1), 1),
            "easy_wasted_pct": round(100 * (route_up & ~hard_te).sum() / max((~hard_te).sum(), 1), 1),
            "accuracy_of_decision": round(100 * (route_up == hard_te).mean(), 1),
        })
    res["coverage_curve_heldout"] = curve
    res["features"] = int(X.shape[1])

    (outdir / "router.json").write_text(
        json.dumps({"params": params.tolist(), "mu": mu.tolist(), "sd": sd.tolist(),
                    "features": int(X.shape[1]), "k3_live": k3_live, "heads": heads,
                    "task_shapes": ["testability", "docstring", "failure", "risk", "coupling"]}, indent=2),
        encoding="utf-8")
    (outdir / "ledger.json").write_text(json.dumps(res, indent=2), encoding="utf-8")
    print(json.dumps(res, indent=2))
    print(f"saved -> {outdir/'router.json'}, {outdir/'ledger.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
