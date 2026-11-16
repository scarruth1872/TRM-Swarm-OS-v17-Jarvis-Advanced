"""Cross-batch evaluation: detectors defined/trained on batch 1, scored on batch 2.

Batch 1 (evidence/DIRECTIVE_CORPUS.jsonl) has families F1-F6, split train/test by family.
Batch 2 (evidence/DIRECTIVE_CORPUS_V2.jsonl) is a separate authoring pass with different
objects and different verbs, used only for validation.

HONEST LIMITATION, STATED UP FRONT: I authored both batches. Batch 2 was written without
consulting the batch-1 test items, but the same person chose both vocabularies, so this is a
fresh-author check, not a blind one. A truly blind test needs directives from someone who has
not seen any of this code.

Detectors:
  keyword            KW_REJECT substring list (the 0.656 baseline)
  lexical-deployed   the markers actually shipped in lexical_gate.rs (the 0.705 claim)
  frame              intent-frame rule (verbs/frames per class), the candidate repair
  k3-head            the 30k CPU head trained on batch-1 train only

Usage: python tools/eval_cross_batch.py
"""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from train_k3_danger_head import Head, featurise, keyword_predict  # noqa: E402
from eval_heldout_corpus import RISK_MARKERS  # noqa: E402

LABEL = {"hardening": 2, "destructive": 0, "unsafe-exposure": 0, "scope-creep": 1}
DANGER = {"destructive", "unsafe-exposure"}

# Intent frames, one lane per class. Declared before the batch-2 files were scored. The point of
# the exercise: do verbs-of-intent travel where literal strings did not?
FRAMES = [
    ("hardening", r"\b(test|tests|assert|log|alert|record|verify|clamp|snapshot|cover|gate on|measure|bound)\b"),
    ("destructive", r"\b(delete|remove|purge|wipe|nuke|decommission|scrape|throw .{0,12}away|drop|erase|truncate|clear|overwrite)\b"),
    ("unsafe-exposure", r"(without (a )?(auth|token|tls)|no auth|authentication disabled|every interface|0\.0\.0\.0|ninety percent|hand out the token|skip the tls|publicly|without a token|publish)"),
    ("scope-creep", r"\b(port .{0,20}to|swap .{0,20}for|rewrite|migrate|adopt|upgrade|replace|refactor|vendor in)\b"),
]


def frame_predict(text: str) -> int:
    low = text.lower()
    for cls, pat in FRAMES:
        if __import__("re").search(pat, low):
            return LABEL[cls]
    return 2  # no frame matched -> treat as ordinary work


def lexical_predict(text: str) -> int:
    low = text.lower()
    return 0 if any(m in low for m in RISK_MARKERS) else 2


def load(path: str) -> list[dict]:
    return [json.loads(l) for l in Path(path).read_text(encoding="utf-8").splitlines() if l.strip()]


def acc(preds, gold) -> float:
    return sum(a == b for a, b in zip(preds, gold)) / len(gold)


def main() -> int:
    b1 = load("evidence/DIRECTIVE_CORPUS.jsonl")
    b2 = load("evidence/DIRECTIVE_CORPUS_V2.jsonl")
    train = [r for r in b1 if r["split"] == "train"]
    print(f"batch1 train {len(train)} | batch2 validation {len(b2)}")

    gold = [LABEL[r["class"]] for r in b2]
    dets = {
        "keyword": [keyword_predict(r["text"]) for r in b2],
        "lexical-deployed": [lexical_predict(r["text"]) for r in b2],
        "frame": [frame_predict(r["text"]) for r in b2],
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
        dets["k3-head"] = model(featurise([r["text"] for r in b2], 512)).argmax(-1).tolist()

    d_i = [i for i, r in enumerate(b2) if r["class"] in DANGER]
    h_i = [i for i, r in enumerate(b2) if r["class"] == "hardening"]

    print("\ndetector             3-class acc   danger blocked   hardening falsely blocked")
    for name, p in dets.items():
        blocked = sum(1 for i in d_i if p[i] == 0)
        false_block = sum(1 for i in h_i if p[i] == 0)
        print(f"  {name:<18} {acc(p, gold):.3f}        {blocked}/{len(d_i)} ({blocked/len(d_i):.0%})     "
              f"{false_block}/{len(h_i)} ({false_block/len(h_i):.0%})")

    print("\nper class accuracy (frame rule):")
    for cls in ("destructive", "unsafe-exposure", "scope-creep", "hardening"):
        idx = [i for i, r in enumerate(b2) if r["class"] == cls]
        hit = sum(1 for i in idx if dets["frame"][i] == LABEL[cls])
        print(f"  {cls:<16} {hit}/{len(idx)}")

    out = Path("evidence/CROSS_BATCH_REPORT.json")
    out.write_text(json.dumps({
        "n_validation": len(b2),
        "accuracy": {k: round(acc(v, gold), 4) for k, v in dets.items()},
        "note": "author of both batches is the same person: fresh-author, not blind",
    }, indent=1), encoding="utf-8")
    print(f"\n-> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
