#!/usr/bin/env python3
"""Free labels: does the 1,000-row load-test corpus predict its own strict-grounding outcome?

No teacher is needed. `scratch/loadtest/run1000_clean/results.jsonl` already records, per task, whether the 1.5B's
answer was schema-valid and whether its citations were real identifiers. That is a binary label with 1,000 examples
attached to prompts this platform actually sends.

What is being tested: the earlier router (`tools/train_router.py`) used 36 handcrafted features and reached
**AUC 0.580** held-out (train 0.680). This asks whether mean-pooled nomic embeddings from `:8593` beat that, using the
same file-level split so no file appears in both halves.

Baselines are stated before the run:
  * majority class (the "always predict the common answer" floor)
  * AUC 0.580 - the handcrafted-feature router already measured
  * AUC 0.500 - no information

Honest scoring rules applied here:
  * SPLIT BY FILE, never by row: two tasks on the same file are near-duplicates, and row-splitting inflates AUC.
  * Report train AUC beside held-out AUC. A big gap is overfitting, not skill.
  * Report the class balance. If the positive rate is 58 %, 58 % accuracy is the floor, not a result.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

import torch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import train_ssm_readout as T  # noqa: E402

ID_OK_LEN = 3


def strict_ok(rec: dict) -> int:
    """Same strict-grounding definition used everywhere else in this repo (G124/G126)."""
    if not rec.get("schema_ok"):
        return 0
    syms = rec.get("symbols") or []
    if len(syms) < 2:
        return 0
    for s in syms:
        if not isinstance(s, str) or len(s) < ID_OK_LEN or not s.replace("_", "a").replace(".", "a").isalnum():
            return 0
    return 1


def build_rows(run: str) -> list[dict]:
    """(prompt text, strict label, file group) rebuilt from the recorded run."""
    import loadtest_tasks as lt
    recs = [json.loads(l) for l in (ROOT / run / "results.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    sources = lt.collect_sources(1000)
    # Reconstruct by the job id the harness recorded. Keying on (file, task) LOSES rows: task rotates with
    # i % len(TASK_ORDER) while the source rotates with i % len(sources), so the same pair recurs and a dict
    # silently keeps only the last - 526 of 1,000 rows were dropped that way on the first attempt.
    rows, missing = [], 0
    for r in recs:
        i = r.get("id")
        if not isinstance(i, int) or i >= 1000:
            missing += 1
            continue
        src = sources[i % len(sources)]
        ex = lt.excerpt_of(src)
        if not ex:
            missing += 1
            continue
        rows.append({"text": f"{lt.TASKS[r['task']]['ask']}\n\n{ex}",
                     "label": strict_ok(r), "group": r["file"], "task": r["task"]})
    if missing:
        print(f"[warn] {missing} records could not be rebuilt")
    return rows


def folds(rows: list[dict], k: int = 5, seed: int = 23) -> list[list[int]]:
    """Group k-fold by FILE: a file's tasks all land in the same fold."""
    buckets: dict[str, list[int]] = {}
    for i, r in enumerate(rows):
        buckets.setdefault(r["group"], []).append(i)
    assign: dict[str, int] = {}
    for g in sorted(buckets):
        h = int(hashlib.sha256(f"{seed}:{g}".encode()).hexdigest()[:8], 16)
        assign[g] = h % k
    return [assign[r["group"]] for r in rows]


def pooled_embed(texts: list[str]) -> torch.Tensor:
    rows = []
    for tx in texts:
        chunks = T.split_chunks(tx)
        vecs = T._embed_batch(chunks) if chunks else []
        proj = [T.project_768(v) for v in vecs]
        proj = [p for p in proj if p is not None]
        if not proj:
            rows.append(torch.zeros(T.D_MODEL))
            continue
        v = torch.stack(proj).mean(dim=0)
        n = v.norm()
        rows.append(v / n if n > 1e-6 else v)
    return torch.stack(rows)


def fit_logreg(X, y, epochs=500, lr=0.05, l2=0.05):
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


def auc(scores: list[float], labels: list[int]) -> float:
    """Rank-based AUC with ties averaged. Returns 0.5 when a class is absent."""
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    if not pos or not neg:
        return 0.5
    wins = 0.0
    for p in pos:
        for n in neg:
            wins += 1.0 if p > n else (0.5 if p == n else 0.0)
    return wins / (len(pos) * len(neg))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", default="scratch/loadtest/run1000_clean")
    ap.add_argument("--out", default="evidence/FREE_LABEL_REPORT.json")
    ap.add_argument("--emit-labels", default="scratch/ssm_free/labels_grounding.jsonl")
    ap.add_argument("--k", type=int, default=5)
    args = ap.parse_args()

    rows = build_rows(args.run)
    labels = [r["label"] for r in rows]
    base = max(sum(labels), len(labels) - sum(labels)) / len(labels)
    print(f"rows {len(rows)} over {len({r['group'] for r in rows})} files | strict-positive {sum(labels)} "
          f"({100*sum(labels)/len(labels):.1f} %) | majority baseline {base:.3f}")

    emit = ROOT / args.emit_labels
    emit.parent.mkdir(parents=True, exist_ok=True)
    with emit.open("w", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps({"topic": r["group"], "role": r["task"], "label": 1 if r["label"] else -1,
                                 "text_chars": len(r["text"])}, ensure_ascii=False) + "\n")
    print(f"emitted trainer-format labels -> {emit} ({len(rows)} rows, topic=file)")

    print("embedding ...", flush=True)
    X = pooled_embed([r["text"] for r in rows])
    y = torch.tensor([float(l) for l in labels])
    print(f"features {tuple(X.shape)}")

    fold = folds(rows, args.k)
    te_scores: list[float] = []
    te_labels: list[int] = []
    train_aucs = []
    # collected INSIDE the fold loop: te_scores is a compacted list, so indexing it by row id later
    # silently pairs each fold's scores with another fold's rows (that reported 0.441 while the
    # per-fold lines said 0.49-0.53).
    fold_aucs: list[float] = []
    fold_accs: list[float] = []
    for f in range(args.k):
        tr = [i for i in range(len(rows)) if fold[i] != f]
        te = [i for i in range(len(rows)) if fold[i] == f]
        w, b = fit_logreg(X[tr], y[tr])
        with torch.no_grad():
            tr_s = (X[tr] @ w + b).tolist()
            te_s = (X[te] @ w + b).tolist()
        train_aucs.append(auc(tr_s, [labels[i] for i in tr]))
        te_scores += te_s
        te_labels += [labels[i] for i in te]
        l_te = [labels[i] for i in te]
        fold_aucs.append(auc(te_s, l_te))
        fold_accs.append(sum(1 for s, l in zip(te_s, l_te) if (s > 0) == (l == 1)) / len(te))
        acc = fold_accs[-1]
        print(f"  fold {f}: train AUC {train_aucs[-1]:.3f} | held-out AUC {auc(te_s, [labels[i] for i in te]):.3f} "
              f"| held-out acc {acc:.3f}  (n={len(te)}, {len({rows[i]['group'] for i in te})} files)")

    # PER-FOLD AUC, then mean: pooling raw logits across folds compares scores fitted by different models,
    # which produced a nonsensical 0.420 "AUC" while every individual fold was above 0.48.
    a = sum(fold_aucs) / len(fold_aucs)
    sd = (sum((x - a) ** 2 for x in fold_aucs) / len(fold_aucs)) ** 0.5
    acc = sum(fold_accs) / len(fold_accs)
    print(f"\n=== EMBEDDING PROBE on free labels ({args.k}-fold by FILE)")
    print(f"  held-out AUC        : {a:.3f} +/- {sd:.3f} (per fold: {[round(x,3) for x in fold_aucs]})")
    print(f"  train AUC (mean)    : {sum(train_aucs)/len(train_aucs):.3f}   <- gap = overfit")
    print(f"  held-out accuracy   : {acc:.3f}")
    print(f"  majority baseline   : {base:.3f}")
    print(f"  handcrafted router  : 0.580 AUC (G120, same file-level split)")
    verdict = ("BEATS handcrafted features" if a > 0.58 + 0.05 else
               ("no better than the 36-feature router" if a > 0.53 else "NO SIGNAL"))
    print(f"  verdict             : {verdict}")
    Path(args.out).write_text(json.dumps({
        "rows": len(rows), "files": len({r['group'] for r in rows}), "k": args.k,
        "positive_rate": round(sum(labels) / len(labels), 4), "majority_baseline": round(base, 4),
        "heldout_auc": round(a, 4), "heldout_auc_sd": round(sd, 4),
        "fold_aucs": [round(x, 4) for x in fold_aucs], "train_auc_mean": round(sum(train_aucs) / len(train_aucs), 4),
        "heldout_accuracy": round(acc, 4), "handcrafted_router_auc": 0.580, "verdict": verdict}, indent=2),
        encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
