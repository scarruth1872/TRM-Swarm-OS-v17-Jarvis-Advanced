#!/usr/bin/env python3
"""Is the embedding signal real, or is the SSM read-out head simply the wrong model?

The fitted read-out is an SSM over 6 chunk-embeddings -> a 128-dim state -> a linear head. That is
D_STATE*D_MODEL*2 + D_MODEL = 4,224 fitted parameters for ~264 training rows per fold. It loses to a keyword
regex (0.733) and to always-majority (0.709) on the same folds, both with hashes (0.462) and with real nomic
embeddings (0.609).

So this probe removes the recurrence and asks the prior question: does a MEAN-POOLED version of the same feature
carry the label at all? Same rows, same noise filter, same leave-one-topic-out folds, same baselines; the only
change is a 128-parameter linear head instead of a 4,224-parameter state-space read-out.

Outcome decides the next move, not the other way round:
  probe_beats_keyword  -> the signal is in the features; replace the head (or freeze B/C) and deploy
  probe_loses          -> embeddings carry no more label information than the word list; drop the read-out for good
"""
from __future__ import annotations

import argparse
import collections
import json
import sys
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import train_ssm_readout as T  # noqa: E402


def pooled(feats: list[str], mode: str) -> torch.Tensor:
    """Mean-pool the recurrence's own inputs to one 128-dim vector per text."""
    rows = []
    for tx in feats:
        if mode == "embed":
            chunks = T.split_chunks(tx)
            vecs = T._embed_batch(chunks) if chunks else []
            proj = [T.project_768(v) for v in vecs]
            proj = [p for p in proj if p is not None]
        else:
            toks = T.tokens(tx)
            proj = [T.token_vector(t) for t in toks]
        if not proj:
            rows.append(torch.zeros(T.D_MODEL))
            continue
        v = torch.stack(proj).mean(dim=0)
        n = v.norm()
        rows.append(v / n if n > 1e-6 else v)
    return torch.stack(rows)


def fit_logreg(X: torch.Tensor, y: torch.Tensor, epochs: int = 400, lr: float = 0.05, l2: float = 0.02):
    w = torch.zeros(X.shape[1], requires_grad=True)
    b = torch.zeros(1, requires_grad=True)
    opt = torch.optim.Adam([w, b], lr=lr)
    lossf = torch.nn.BCEWithLogitsLoss()
    for _ in range(epochs):
        opt.zero_grad()
        loss = lossf(X @ w + b, y) + l2 * w.pow(2).sum()
        loss.backward()
        opt.step()
    return w.detach(), float(b.detach())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--labels", default="evidence/SSM_LABELS.jsonl")
    ap.add_argument("--features", choices=["hash", "embed", "both"], default="embed")
    ap.add_argument("--out", default="evidence/SSM_PROBE_REPORT.json")
    args = ap.parse_args()

    rows = [json.loads(l) for l in Path(args.labels).read_text(encoding="utf-8").splitlines() if l.strip()]
    before = len(rows)
    rows = [r for r in rows if not T.is_noise(r["topic"])]
    rows = [r for r in rows if int(r["label"]) != 0]      # the binary head only fits +/-1
    print(f"noise filter: {before} -> {len(rows)} rows | topics {len({r['topic'] for r in rows})}")

    texts = [f"{r['topic']}\n{r.get('role') or ''}" for r in rows]
    labels = torch.tensor([1.0 if int(r["label"]) > 0 else 0.0 for r in rows])
    topics = [r["topic"] for r in rows]

    blocks = []
    modes = ["hash", "embed"] if args.features == "both" else [args.features]
    for m in modes:
        print(f"building {m} features ...", flush=True)
        blocks.append(pooled(texts, m))
    X = torch.cat(blocks, dim=1) if len(blocks) > 1 else blocks[0]
    print(f"feature matrix: {tuple(X.shape)}")

    kw = sum(1 for tx, l in zip(texts, labels.tolist()) if T.keyword_predict(tx) == (1 if l > 0 else -1)) / len(rows)
    print(f"keyword heuristic : {kw:.3f}")

    correct = base = 0
    n = 0
    for held in sorted(set(topics)):
        tr = [i for i, t in enumerate(topics) if t != held]
        te = [i for i, t in enumerate(topics) if t == held]
        if not tr or not te:
            continue
        w, b = fit_logreg(X[tr], labels[tr])
        pred = (X[te] @ w + b > 0).float()
        correct += int((pred == labels[te]).sum())
        maj = collections.Counter(labels[i].item() for i in tr).most_common(1)[0][0]
        base += int((labels[te] == maj).sum())
        n += len(te)

    acc, maj_acc = correct / n, base / n
    print(f"\n=== LINEAR PROBE on mean-pooled {args.features} features ({len(set(topics))} topics, {n} votes)")
    print(f"  probe accuracy    : {acc:.3f}")
    print(f"  majority baseline : {maj_acc:.3f}")
    print(f"  keyword baseline  : {kw:.3f}")
    print(f"  lead vs majority  : {acc - maj_acc:+.3f}   (SSM read-out: -0.100)")
    print(f"  lead vs keyword   : {acc - kw:+.3f}   (SSM read-out: -0.124)")
    verdict = "SIGNAL PRESENT - head is the problem" if acc - kw > 0.05 else "NO SIGNAL BEYOND THE WORD LIST"
    print(f"  verdict           : {verdict}")
    Path(args.out).write_text(json.dumps({
        "features": args.features, "rows": n, "topics": len(set(topics)),
        "probe_accuracy": round(acc, 4), "majority_baseline": round(maj_acc, 4), "keyword_baseline": round(kw, 4),
        "lead_vs_majority": round(acc - maj_acc, 4), "lead_vs_keyword": round(acc - kw, 4),
        "ssm_readout_lead_vs_keyword": -0.124, "verdict": verdict}, indent=2), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
