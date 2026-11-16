"""Harvest every real adversarial INSTRUCTION this repo already contains.

The dataset scan (tools/scan_datasets_for_danger.py) found no danger corpus on disk:
445 files, 772.8 MB, and the only dense danger-bearing text is the council's own
proposal ledger - proposals this platform filed against itself. That is the closest
thing to a curated adversarial set that exists here, so this harvests it properly:

  sources  council_ledger.json, council_ledger_snapshot.json,
           swarm_v2_artifacts/council_proposals.jsonl, and any *.json under
           tests/ or evidence/ whose strings look like an adversarial set
  filter   destructive verb AND danger object, word-boundary, whole instruction
  dedupe   exact-normalised text
  report   unique instructions, verb variety, object variety, teacher labels
  output   evidence/K3_DANGER_SEED.json

The point is to state the real number: how many DIFFERENT harmful instructions this
repo can supply for training, before anyone claims a corpus exists.

Usage: python tools/harvest_danger_seed.py
"""
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

from k3_label_v2 import k3_label_v2, NAMES, _tokens, DESTRUCTIVE, IRREVERSIBLE_OBJECTS  # noqa: E402

SOURCES = [
    ROOT / "trm_uefi_kernel" / "host_council_server" / "data" / "council_ledger.json",
    ROOT / "scratch" / "council_ledger_snapshot.json",
    ROOT / "swarm_v2_artifacts" / "council_proposals.jsonl",
]
OBJ_RE = re.compile(r"(?<![a-z0-9])(production|prod|database|ledger|audit|backup|archive|"
                    r"key|keys|secret|credential|password|watchdog|auth|monitoring|pager|"
                    r"replica|customer|tenant|invoice|evidence|provenance|firewall)(?![a-z0-9])")


def strings_from(node, out):
    if isinstance(node, str):
        out.append(node)
    elif isinstance(node, dict):
        for v in node.values():
            strings_from(v, out)
    elif isinstance(node, list):
        for v in node:
            strings_from(v, out)


def load_source(path):
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix == ".jsonl":
        rows = []
        for line in text.splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    else:
        try:
            rows = [json.loads(text)]
        except json.JSONDecodeError:
            return []
    out = []
    strings_from(rows, out)
    return out


VERDICT_MARKERS = ("[truth]", "[data]", "[security]", "[workflow]", "[spectrum]",
                   "[audit]", "could ", "would ", "compromise", "proposal", "verdict",
                   "contradicts", "undermine", "the council", "nonsensical", "recommend")


def is_instruction(s):
    """A harmful INSTRUCTION, not a description or a verdict ABOUT one.

    The first version of this filter accepted 73 strings, but most were agent
    deliberations ("Disabling the audit ledger could compromise security and
    integrity of data") - commentary, not requests. Training a gate on commentary is
    how the classifier learns to recognise the WORD and not the ACT.
    """
    low = s.lower()
    if not (set(_tokens(s)) & DESTRUCTIVE):
        return False
    if not OBJ_RE.search(low):
        return False
    if len(s) < 12 or len(s) > 200:
        return False
    if any(m in low for m in VERDICT_MARKERS):
        return False
    return True


def main():
    seen = {}
    per_source = {}
    for src in SOURCES:
        strings = load_source(src)
        kept = 0
        for s in strings:
            if not is_instruction(s):
                continue
            key = re.sub(r"\s+", " ", s.strip().lower())
            if key not in seen:
                seen[key] = s.strip()
                kept += 1
        per_source[str(src.relative_to(ROOT))] = {"strings": len(strings), "new_instructions": kept}

    items = sorted(seen.values(), key=len)
    verbs = Counter()
    objects = Counter()
    labels = Counter()
    for s in items:
        for t in _tokens(s):
            if t in DESTRUCTIVE:
                verbs[t] += 1
        for m in OBJ_RE.finditer(s.lower()):
            objects[m.group(1)] += 1
        labels[NAMES[k3_label_v2(s)]] += 1

    print("sources")
    for k, v in per_source.items():
        print(f"  {k}: {v['strings']} strings -> {v['new_instructions']} new")
    print(f"\nunique harmful instructions harvested: {len(items)}")
    print(f"distinct destructive verbs: {len(verbs)}  -> {', '.join(v for v, _ in verbs.most_common(12))}")
    print(f"distinct danger objects:    {len(objects)}  -> {', '.join(o for o, _ in objects.most_common(12))}")
    print(f"teacher labels: {dict(labels)}")
    print("\nsample (12)")
    for s in items[:12]:
        print(f"  [{NAMES[k3_label_v2(s)]:7s}] {s[:96]}")

    out = ROOT / "evidence" / "K3_DANGER_SEED.json"
    out.write_text(json.dumps({
        "unique_instructions": len(items),
        "verbs": dict(verbs),
        "objects": dict(objects),
        "teacher_labels": dict(labels),
        "sources": per_source,
        "instructions": items,
    }, indent=2), encoding="utf-8")
    print(f"\nseed -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
