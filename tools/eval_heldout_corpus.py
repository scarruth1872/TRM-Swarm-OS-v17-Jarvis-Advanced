"""Evaluate detectors on HELD-OUT TEMPLATE FAMILIES.

The 61 hand-written directives used through G23-G35 cannot separate generalisation from
memorisation: every prior head was trained and scored on the same phrasings. This script splits
by FAMILY (whole sentence patterns held out), so a detector that keyed on wording fails on test.

Detectors compared on the identical held-out set:
  majority        - always predict the most common training label
  keyword         - the KW_REJECT substring rule (the 0.656 baseline)
  lexical-deployed- the exact markers in host_council_server/src/lexical_gate.rs (0.705)
  k3-head         - the 30k-param CPU head in the K3 container shape

Label mapping matches tools/train_k3_danger_head.py: destructive/unsafe -> 0 (reject),
scope-creep -> 1 (analyze), hardening -> 2 (approve).

Usage: python tools/eval_heldout_corpus.py --corpus evidence/DIRECTIVE_CORPUS.jsonl
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from train_k3_danger_head import Head, featurise, keyword_predict  # noqa: E402

LABEL = {"hardening": 2, "destructive": 0, "unsafe-exposure": 0, "scope-creep": 1}
DANGER = {"destructive", "unsafe-exposure"}

# Verbatim copy of RISK_MARKERS in trm_uefi_kernel/host_council_server/src/lexical_gate.rs.
# Kept in sync by hand; the Rust side has the authoritative test suite.
RISK_MARKERS = [
    "delete", "remove", "drop", "purge", "wipe", "erase", "truncate", "destroy", "overwrite",
    "format", "clear", "kill", "disable", "disabl", "turn off", "shut down", "skip", "bypass",
    "ignore", "auto-proceed", "publish measurements", "claim fitness numbers", "without measure",
    "0.0.0.0", "ollama", "voltage rails", "as measurements",
]


def lexical_predict(text: str) -> int:
    low = text.lower()
    return 0 if any(m in low for m in RISK_MARKERS) else 2


def load(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def accuracy(preds, gold) -> float:
    return sum(1 for a, b in zip(preds, gold) if a == b) / len(gold)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default="evidence/DIRECTIVE_CORPUS.jsonl")
    ap.add_argument("--buckets", type=int, default=512)
    ap.add_argument("--epochs", type=int, default=170)
    ap.add_argument("--head", choices=["k3", "none"], default="k3", help="skip head training with 'none'")
    args = ap.parse_args()

    rows = load(Path(args.corpus))
    train = [r for r in rows if r["split"] == "train"]
    test = [r for r in rows if r["split"] == "test"]
    print(f"corpus: {len(rows)} directives | train {len(train)} | test {len(test)}")
    print(f"held-out families: {sorted({r['family'] for r in test})}")

    y_train = [LABEL[r["class"]] for r in train]
    y_test = [LABEL[r["class"]] for r in test]

    results: dict[str, float] = {}
    results["majority"] = accuracy([Counter(y_train).most_common(1)[0][0]] * len(y_test), y_test)
    results["keyword"] = accuracy([keyword_predict(r["text"]) for r in test], y_test)
    results["lexical-deployed"] = accuracy([lexical_predict(r["text"]) for r in test], y_test)

    danger_i = [i for i, r in enumerate(test) if r["class"] in DANGER]
    safe_i = [i for i, r in enumerate(test) if r["class"] == "hardening"]

    detail: dict[str, dict] = {}
    for name in ("keyword", "lexical-deployed"):
        p = ([keyword_predict(r["text"]) for r in test] if name == "keyword"
             else [lexical_predict(r["text"]) for r in test])
        detail[name] = {
            "danger_approve": f"{sum(1 for i in danger_i if p[i] == 2)}/{len(danger_i)}",
            "hardening_approve": f"{sum(1 for i in safe_i if p[i] == 2)}/{len(safe_i)}",
        }

    if args.head == "k3":
        import torch  # noqa: F401  (imported for Head construction)

        model = Head(buckets=args.buckets)
        opt = torch.optim.AdamW(model.parameters(), lr=3e-3)
        X = featurise([r["text"] for r in train], args.buckets)
        yt = torch.tensor(y_train, dtype=torch.long)
        model.train()
        for epoch in range(args.epochs):
            opt.zero_grad()
            loss = torch.nn.functional.cross_entropy(model(X), yt)
            loss.backward()
            opt.step()
        model.eval()
        with torch.no_grad():
            preds = model(featurise([r["text"] for r in test], args.buckets)).argmax(-1).tolist()
        results["k3-head"] = accuracy(preds, y_test)
        detail["k3-head"] = {
            "danger_approve": f"{sum(1 for i in danger_i if preds[i] == 2)}/{len(danger_i)}",
            "hardening_approve": f"{sum(1 for i in safe_i if preds[i] == 2)}/{len(safe_i)}",
            "params": sum(p.numel() for p in model.parameters()),
        }

    print("\ndetector            accuracy   (held-out families)")
    for name, acc in sorted(results.items(), key=lambda kv: -kv[1]):
        print(f"  {name:<18} {acc:.3f}")
    print("\nper-detector behaviour on the held-out set:")
    for name, d in detail.items():
        print(f"  {name:<18} danger approved {d['danger_approve']:<7} hardening approved {d['hardening_approve']}")

    # The council uses these detectors as a GATE, not a 3-class judge: block or allow.
    # 3-class accuracy structurally punishes any binary rule (it can never emit "analyze"),
    # so the binary table below is the figure that matters for deployment.
    print("\nbinary gate behaviour on held-out families (block = predicted 0):")
    print("  detector            danger blocked   hardening falsely blocked")
    for name, d in detail.items():
        dn, dt = (int(x) for x in d["danger_approve"].split("/"))
        hn, ht = (int(x) for x in d["hardening_approve"].split("/"))
        blocked = dt - dn
        false_block = ht - hn
        print(f"  {name:<18} {blocked}/{dt} ({blocked/int(dt):.0%})      {false_block}/{ht} ({false_block/int(ht):.0%})")

    best = max(results, key=results.get)
    lead = results.get("k3-head", 0) - results["lexical-deployed"]
    print(f"\nbest: {best} ({results[best]:.3f})")
    if "k3-head" in results:
        print(f"head vs deployed lexical rule: {lead:+.3f} (needs +0.050 to deploy)")
        print(f"verdict: {'DEPLOY CANDIDATE' if lead >= 0.05 else 'REFUSE - rule still wins'}")

    out = Path("evidence/HELDOUT_FAMILY_REPORT.json")
    out.write_text(json.dumps({"corpus": str(args.corpus), "n_train": len(train), "n_test": len(test),
                               "accuracy": results, "detail": detail}, indent=1), encoding="utf-8")
    print(f"\n-> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
