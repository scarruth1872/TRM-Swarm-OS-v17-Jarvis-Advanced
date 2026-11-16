"""Blind evaluation: detectors scored on directives written by an independent author.

WHY THIS FILE EXISTS
--------------------
Every previous corpus (DIRECTIVE_CORPUS, DIRECTIVE_CORPUS_V2) was written by the same hand
that wrote the detectors, so the 0.927 frame figure and the 0.705 lexical figure were
fresh-author numbers at best. This set was produced by a separate agent that was explicitly
told NOT to read trm_uefi_kernel/host_council_server/src/ (where the gates live), given only
the danger/safe definitions. That makes it the first set where the author could not have
seen the rule they are being scored against.

Binary task: should this directive be blocked?  danger = block, safe = do not block.
    danger blocked        = recall on danger  (higher is better)
    safe falsely blocked  = false-positive rate on legitimate work (lower is better)
A detector is only useful if it moves recall up WITHOUT dragging the false-block rate up.

Usage: python tools/eval_blind.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from eval_cross_batch import FRAMES, LABEL, frame_predict, lexical_predict  # noqa: E402
from train_k3_danger_head import Head, featurise, keyword_predict  # noqa: E402

BLIND = "evidence/BLIND_DIRECTIVES.jsonl"
BATCH1 = "evidence/DIRECTIVE_CORPUS.jsonl"


def load(path: str) -> list[dict]:
    return [json.loads(l) for l in Path(path).read_text(encoding="utf-8").splitlines() if l.strip()]


def normalise(text: str) -> str:
    """Mirror of trm_uefi_kernel/host_council_server/src/topic_norm.rs (address frames only).

    Kept deliberately small: if the Rust normaliser and this copy disagree, the copy is wrong.
    """
    frames = [
        "ask the council whether we should ", "ask the council whether ", "ask the council if ",
        "ask the council about ", "ask the council ",
        "consult the council on whether we should ", "consult the council on ", "consult the council about ",
        "take this to the council: ", "bring this to the council: ", "put it to the council: ",
        "council vote on ", "council decision on ", "proposal: ", "propose that we ",
        "i want to ", "i would like to ", "we want to ", "we would like to ",
        "i think we should ", "i think we can ", "we should ", "should we ", "can we ", "could we ",
        "please, ", "please ", "let's ", "lets ", "uh, ", "um, ", "ok so, ", "ok, ", "so, ",
    ]
    cur = text.strip().rstrip(".?!").strip()
    for _ in range(4):
        low = cur.lower()
        hit = next((f for f in frames if low.startswith(f)), None)
        if not hit:
            break
        rest = cur[len(hit):].strip().rstrip(".?!").strip()
        if not rest:
            break
        cur = rest
    return cur


def main() -> int:
    rows = load(BLIND)
    gold = [1 if r["label"] == "danger" else 0 for r in rows]
    texts = [r["text"] for r in rows]
    print(f"blind set: {len(rows)} directives ({sum(gold)} danger / {len(gold)-sum(gold)} safe)")
    print(f"author: independent subagent, gate source not shown\n")

    train = [r for r in load(BATCH1) if r["split"] == "train"]
    dets = {
        "never block (majority)": [0] * len(rows),
        "keyword rule": [1 if keyword_predict(t) == 0 else 0 for t in texts],
        "lexical-deployed": [1 if lexical_predict(t) == 0 else 0 for t in texts],
        "frame lane": [1 if frame_predict(t) == 0 else 0 for t in texts],
        "frame + normalise": [1 if frame_predict(normalise(t)) == 0 else 0 for t in texts],
        "always block": [1] * len(rows),
    }

    import torch

    model = Head(buckets=512)
    opt = torch.optim.AdamW(model.parameters(), lr=3e-3)
    X = featurise([r["text"] for r in train], 512)
    yt = torch.tensor([LABEL[r["class"]] for r in train], dtype=torch.long)
    model.train()
    for _ in range(170):
        opt.zero_grad()
        torch.nn.functional.cross_entropy(model(X), yt).backward()
        opt.step()
    model.eval()
    with torch.no_grad():
        pred = model(featurise(texts, 512)).argmax(-1).tolist()
    dets["k3-head"] = [1 if p == 0 else 0 for p in pred]

    n_d = sum(gold)
    n_s = len(gold) - n_d
    print(f"{'detector':<24} {'danger blocked':>15} {'safe falsely blocked':>21} {'balanced':>9}")
    print("-" * 74)
    report = {}
    for name, p in dets.items():
        tb = sum(1 for i in range(len(rows)) if gold[i] == 1 and p[i] == 1)
        fb = sum(1 for i in range(len(rows)) if gold[i] == 0 and p[i] == 1)
        rec, fpr = tb / n_d, fb / n_s
        bal = (rec + (1 - fpr)) / 2
        print(f"  {name:<22} {tb:>3}/{n_d} ({rec:>4.0%})   {fb:>3}/{n_s} ({fpr:>4.0%})       {bal:.3f}")
        report[name] = {"danger_blocked": tb, "danger_total": n_d,
                        "safe_false_blocked": fb, "safe_total": n_s,
                        "recall": round(rec, 4), "false_block_rate": round(fpr, 4),
                        "balanced_acc": round(bal, 4)}

    # per-item view for the frame lane: what it misses and what it wrongly blocks
    print("\nframe lane errors:")
    p = dets["frame + normalise"]
    for i, r in enumerate(rows):
        if p[i] != gold[i]:
            kind = "MISSED danger" if gold[i] == 1 else "FALSE block"
            print(f"  {kind:<14} {r['text'][:78]}")

    out = Path("evidence/BLIND_REPORT.json")
    out.write_text(json.dumps({
        "n": len(rows), "danger": n_d, "safe": n_s,
        "author": "independent subagent; gate source not shown",
        "detectors": report,
    }, indent=1), encoding="utf-8")
    print(f"\n-> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
