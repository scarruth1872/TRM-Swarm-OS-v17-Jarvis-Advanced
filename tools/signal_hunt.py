#!/usr/bin/env python3
"""Signal hunt: does ANY corpus on this box have text that predicts its label?

Three Mamba-3 experiments have now returned null (council read-out 0.609 vs 0.733 keyword; pooled probe 0.727 ==
majority; grounding 0.511 vs 0.614). Before porting an architecture, establish whether the absence of signal is a
property of the CORPUS or of the harness. So this runs candidate corpora and two controls:

  POSITIVE CONTROL  a label that is literally in the text (task shape in the load-test prompt). A working harness must
                    score near 1.00 here. If it does not, every null result elsewhere is suspect.
  NULL CONTROL      the same texts with shuffled labels. A working harness must score ~0.50 here. Anything well above
                    0.50 is leakage, not signal.

Scoring rules, stated before the run:
  * 5-fold CV, grouped where a group would leak (council proposals repeat across agents; K3 templates repeat across a tag)
  * report mean per-fold AUC +/- sd, never pooled logits from different folds
  * report the majority baseline for every corpus
  * a corpus "has signal" only if mean AUC >= 0.65 -- far above what any of the three null results reached
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import random
import sys
from pathlib import Path

import torch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import train_ssm_readout as T  # noqa: E402

K_FOLDS = 5


def corpora() -> dict:
    """Every candidate, as (text, label 0/1, group) builders."""
    def k3_danger():
        # V2 carries action_id: the semantic identity shared by every framing of one act. Grouping by it makes an
        # UNSEEN-ACTION held-out test possible, which the tag-grouped original could not do (G133).
        path = ROOT / "evidence/K3_SYNTH_CORPUS_V2.jsonl"
        if not path.exists():
            path = ROOT / "evidence/K3_SYNTH_CORPUS.jsonl"
        rows = [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
        print(f"  [k3] source {path.name}")
        out, seen = [], set()
        for r in rows:
            tx = (r.get("text") or "").strip()
            if not tx or tx in seen:
                continue
            seen.add(tx)
            # label 0 is the danger class in this corpus ("Delete the production database" -> tag danger:plain)
            # Prefer action_id; fall back to the full tag (12 values). Splitting the tag on ":" first collapsed it
            # to 3 groups, making 5-fold degenerate and leave-one-tag-out meaningless.
            grp = r.get("action_id") or str(r.get("tag") or "?")
            out.append((tx, 1 if str(r.get("label")) == "0" else 0, str(grp)))
        return out

    def council_4b():
        rows = [json.loads(l) for l in (ROOT / "evidence/SSM_LABELS_4B.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
        out, seen = [], set()
        for r in rows:
            lb = int(r.get("label") or 0)
            if lb == 0:
                continue
            tx = f"{r.get('topic') or ''}\n{r.get('role') or ''}".strip()
            key = (tx, r.get("proposal_id"))
            if not tx or key in seen:
                continue
            seen.add(key)
            # group by proposal: one proposal is voted on by many agents, so its topic text repeats
            out.append((tx, 1 if lb > 0 else 0, str(r.get("proposal_id"))))
        return out

    def grounding():
        p = ROOT / "scratch/ssm_free/labels_grounding.jsonl"
        rows = [json.loads(l) for l in p.read_text(encoding="utf-8").splitlines() if l.strip()]
        return [((f"{r['role']}\n"), 1 if r["label"] > 0 else 0, r["topic"]) for r in rows]

    def control_shape():
        """POSITIVE CONTROL: the label is in the prompt text by construction."""
        import loadtest_tasks as lt
        sources = lt.collect_sources(1000)
        out = []
        for i in range(600):
            src = sources[i % len(sources)]
            ex = lt.excerpt_of(src)
            if not ex:
                continue
            task = lt.TASK_ORDER[i % len(lt.TASK_ORDER)]
            out.append((f"{lt.TASKS[task]['ask']}\n\n{ex}", 1 if task == "risk" else 0, str(src.relative_to(ROOT))))
        return out

    # grounding_free is deliberately NOT in the hunt: labels_grounding.jsonl stores no excerpt, so only the role
    # string would be embedded. It was already measured properly with the real prompts (0.511, G131).
    def k3_scope():
        """THE HARD TEST: destructive verb against the SAME destructive verb, differing only in object class.

        "Delete the production database" vs "Delete the scratch directory". Both contain a destructive verb, neither
        carries a negation or a polite frame, so a keyword rule that fires on "delete" cannot separate them and a model
        that learns the prefix "Do not" gets nothing. This is the axis the K3 gate exists for (G25: the gate beat a
        keyword list by 0.635 only on hashed features)."""
        path = ROOT / "evidence/K3_SYNTH_CORPUS_V2.jsonl"
        if not path.exists():
            path = ROOT / "evidence/K3_SYNTH_CORPUS.jsonl"
        rows = [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
        out, seen = [], set()
        for r in rows:
            tag = str(r.get("tag") or "")
            tx = (r.get("text") or "").strip()
            if tx in seen:
                continue
            if tag.startswith("danger:plain"):
                out.append((tx, 1, str(r.get("action_id") or tag)))    # production target, bare command
            elif tag == "twin:scope":
                out.append((tx, 0, str(r.get("action_id") or tag)))    # same verb, ephemeral target
        return out

    return {"k3_danger": k3_danger, "k3_scope": k3_scope,
            "council_4b": council_4b, "CONTROL_shape": control_shape}


_EMB_CACHE: dict[str, torch.Tensor] = {}


def pooled(texts: list[str]) -> torch.Tensor:
    cache = _EMB_CACHE
    out = []
    for i, tx in enumerate(texts):
        v = cache.get(tx)
        if v is None:
            chunks = T.split_chunks(tx)
            vecs = T._embed_batch(chunks) if chunks else []
            proj = [T.project_768(x) for x in vecs]
            proj = [p for p in proj if p is not None]
            if proj:
                m = torch.stack(proj).mean(dim=0)
                n = m.norm()
                v = m / n if n > 1e-6 else m
            else:
                v = torch.zeros(T.D_MODEL)
            cache[tx] = v
        out.append(v)
        if (i + 1) % 250 == 0:
            print(f"    embedded {i+1}/{len(texts)}", flush=True)
    return torch.stack(out)


def fold_assign(groups: list[str], k: int, seed: int = 23) -> list[int]:
    a = {}
    for g in sorted(set(groups)):
        a[g] = int(hashlib.sha256(f"{seed}:{g}".encode()).hexdigest()[:8], 16) % k
    return [a[g] for g in groups]


def auc(scores, labels) -> float:
    pos = [s for s, l in zip(scores, labels) if l == 1]
    neg = [s for s, l in zip(scores, labels) if l == 0]
    if not pos or not neg:
        return 0.5
    w = sum(1.0 if p > n else (0.5 if p == n else 0.0) for p in pos for n in neg)
    return w / (len(pos) * len(neg))


def fit(X, y, epochs=500, lr=0.05, l2=0.05):
    w = torch.zeros(X.shape[1], requires_grad=True)
    b = torch.zeros(1, requires_grad=True)
    opt = torch.optim.Adam([w, b], lr=lr)
    f = torch.nn.BCEWithLogitsLoss()
    for _ in range(epochs):
        opt.zero_grad()
        (f(X @ w + b, y) + l2 * w.pow(2).sum()).backward()
        opt.step()
    return w.detach(), float(b.detach())


def run(name: str, rows: list[tuple[str, int, str]], k: int = K_FOLDS, fold_mode: str = "group") -> dict:
    texts = [r[0] for r in rows]
    labels = [r[1] for r in rows]
    groups = [r[2] for r in rows]
    pos = sum(labels)
    base = max(pos, len(labels) - pos) / len(labels)
    print(f"\n=== {name}: {len(rows)} rows, {len(set(groups))} groups, {100*pos/len(labels):.1f} % positive")
    X = pooled(texts)
    y = torch.tensor([float(l) for l in labels])
    if fold_mode == "group-loto":
        # one fold per group: with 12 twin/danger tags, this holds out an entire phrasing family each time
        order = {g: i for i, g in enumerate(sorted(set(groups)))}
        fold = [order[g] for g in groups]
        k = len(order)
    elif fold_mode == "random":
        # Used only where the group key has too few distinct values for k folds. Random folds on twin-phrased data
        # risk near-duplicate leakage ACROSS folds, so the number is an upper bound and is labelled as such.
        rnd = random.Random(23)
        fold = [rnd.randrange(k) for _ in rows]
    else:
        fold = fold_assign(groups, k)
    aucs = []
    for f in range(k):
        tr = [i for i in range(len(rows)) if fold[i] != f]
        te = [i for i in range(len(rows)) if fold[i] == f]
        if not tr or not te or len({labels[i] for i in te}) < 2:
            continue
        w, b = fit(X[tr], y[tr])
        with torch.no_grad():
            s = (X[te] @ w + b).tolist()
        a = auc(s, [labels[i] for i in te])
        aucs.append(a)
        print(f"  fold {f}: n={len(te)}  AUC {a:.3f}")
    m = sum(aucs) / len(aucs) if aucs else 0.5
    sd = (sum((x - m) ** 2 for x in aucs) / len(aucs)) ** 0.5 if aucs else 0.0
    verdict = ("SIGNAL" if m >= 0.65 else ("WEAK SIGNAL" if m >= 0.55 else "NO SIGNAL"))
    print(f"  -> mean AUC {m:.3f} +/- {sd:.3f} | majority {base:.3f} | {verdict}")
    return {"corpus": name, "rows": len(rows), "groups": len(set(groups)), "positive_rate": round(pos / len(labels), 4),
            "majority_baseline": round(base, 4), "mean_auc": round(m, 4), "sd": round(sd, 4),
            "fold_aucs": [round(x, 4) for x in aucs], "verdict": verdict}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="evidence/SIGNAL_HUNT.json")
    ap.add_argument("--only", default=None, help="run one corpus by name")
    ap.add_argument("--fold-mode", choices=["group", "group-loto", "random"], default="group")
    args = ap.parse_args()

    results = []
    builders = corpora()
    names = [args.only] if args.only else list(builders)
    for n in names:
        rows = builders[n]()
        if not rows:
            print(f"\n=== {n}: empty"); continue
        results.append(run(n, rows, fold_mode=args.fold_mode))

    # NULL CONTROL: same texts as the strongest candidate, labels shuffled. Must land near 0.50.
    k3 = builders["k3_danger"]()
    shuffled = [r[1] for r in k3]
    random.Random(7).shuffle(shuffled)
    results.append(run("CONTROL_shuffled", [(k3[i][0], shuffled[i], k3[i][2]) for i in range(len(k3))]))

    # A partial run must NOT overwrite the canonical artifact: an --only run replaced the four-corpus record with
    # two corpora, leaving the docs citing numbers the evidence file no longer contained.
    out_path = Path(args.out)
    if args.only:
        out_path = out_path.with_name(f"{out_path.stem}_{args.only}.json")
        print(f"[partial run] writing {out_path.name}; canonical {Path(args.out).name} untouched")
    out_path.write_text(json.dumps({"results": results,
        "rule": "signal requires mean AUC >= 0.65; controls must show CONTROL_shape high and CONTROL_shuffled ~0.50",
        "partial": bool(args.only), "fold_mode": args.fold_mode},
        indent=2), encoding="utf-8")
    print("\n=== SUMMARY ===")
    for r in sorted(results, key=lambda x: -x["mean_auc"]):
        print(f"  {r['corpus']:<18} AUC {r['mean_auc']:.3f} +/- {r['sd']:.3f}  majority {r['majority_baseline']:.3f}  {r['verdict']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
