"""Why did the linear read-out score AUC 0.500 on corpus where the probe scored 0.670?

Compares the two fold schemes on identical rows and prints the score range per fold, so a constant-output bug is
distinguishable from a genuinely uninformative head.
"""
import collections
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import train_ssm_readout as T  # noqa: E402

T.FEATURE_MODE = "embed"
ROOT = Path(__file__).resolve().parents[1]


def auc(scores, labels):
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    if not pos or not neg:
        return 0.5
    return sum(1.0 if p > n else (0.5 if p == n else 0.0) for p in pos for n in neg) / (len(pos) * len(neg))


rows = [json.loads(l) for l in (ROOT / "evidence/SSM_LABELS_4B.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
rows = [r for r in rows if not T.is_noise(r["topic"]) and int(r["label"]) != 0]
texts = [f"{r['topic']}\n{r.get('role') or ''}" for r in rows]
labels = [1 if int(r["label"]) > 0 else 0 for r in rows]
topics = [r["topic"] for r in rows]
props = [str(r.get("proposal_id")) for r in rows]

by = collections.defaultdict(list)
for t, l in zip(topics, labels):
    by[t].append(l)
mixed = sum(1 for v in by.values() if len(set(v)) > 1)
print(f"rows {len(rows)} | topics {len(by)} | proposals {len(set(props))} | positive {sum(labels)}")
print(f"topics with BOTH labels: {mixed} of {len(by)}   <- if ~0, a topic-level hold-out cannot score per-vote ranking")

X = T.pooled_matrix(texts)
print(f"X {tuple(X.shape)} | row-norm mean {float(X.norm(dim=1).mean()):.3f}")


def run(groups, name):
    uniq = sorted(set(groups))
    fold_aucs, ranges = [], []
    for g in uniq:
        tr = [i for i, x in enumerate(groups) if x != g]
        te = [i for i, x in enumerate(groups) if x == g]
        if not tr or not te:
            continue
        p = T.fit_linear(X[tr], [1 if labels[i] else -1 for i in tr], epochs=500)
        if p is None:
            continue
        _, proj = T.predict_linear(X[te], p, band=0.25)
        # PER-FOLD AUC: pooling logits from differently calibrated folds mixes scales (this reported 0.329
        # while every individual fold was near or above 0.5).
        fold_aucs.append(auc(proj, [labels[i] for i in te]))
        ranges.append(round(max(proj) - min(proj), 3))
    mean = sum(fold_aucs) / len(fold_aucs)
    sd = (sum((x - mean) ** 2 for x in fold_aucs) / len(fold_aucs)) ** 0.5
    print(f"  {name}: AUC {mean:.3f} +/- {sd:.3f} | score spread per fold min/med/max "
          f"{min(ranges)}/{sorted(ranges)[len(ranges)//2]}/{max(ranges)} | folds {len(ranges)}")


run(topics, "LOTO by topic (trainer default)")
run(props, "grouped by proposal (probe scheme)")
run([t[:40] for t in topics], "LOTO by topic[:40] (truncated key, as used in reporting)")
