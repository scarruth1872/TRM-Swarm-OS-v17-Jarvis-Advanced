#!/usr/bin/env python3
"""ledger_contradictions.py — query the sealed event ledger for blocks that contradict themselves.

WHY THIS EXISTS
---------------
G88 measured 147 sealed blocks in `system_event_ledger.jsonl` whose `phoenix_status` is CONFIRMED while
their own `severity` is WARNING and their event text says "offline". Their hash chain is intact — every
one of 1,000 sampled hashes rederived — which makes the contradiction *reproducible*, not benign: the
block says something it does not mean, and the chain certifies only that nobody edited it afterwards.

The writer was fixed (status is derived from content when not passed explicitly). The 147 existing
records are NOT rewritten: re-sealing would rewrite history. Instead they are queryable here, so the
errata can be audited without re-reading the file.

USAGE
-----
    python tools/ledger_contradictions.py                 # summary + first 10
    python tools/ledger_contradictions.py --json          # machine-readable list
    python tools/ledger_contradictions.py --all           # every contradiction
    python tools/ledger_contradictions.py --since 32500   # only seq >= 32500

WHAT IT DOES NOT DO
-------------------
It does not verify the hash chain (that is `scripts/regen_phoenix_manifest.py` / the audit module) and it
does not repair anything. A contradiction found here is evidence, not a to-do.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

DEFAULT_LEDGER = Path(__file__).resolve().parent.parent / "swarm_v2_global_memory" / "system_event_ledger.jsonl"


def derived_status(rec: dict) -> str:
    """The audit's own rule (audit_legacy_event_stream.py): WARNING severity or an offline event is PARTIAL.

    Kept identical to the writer's fixed default so this tool and the writer cannot drift apart.
    """
    severity = str(rec.get("severity", "INFO")).upper()
    event = str(rec.get("event", "")).lower()
    if severity == "WARNING" or "offline" in event:
        return "PARTIAL"
    return "CONFIRMED"


def find_contradictions(path: Path, since: int | None = None) -> list[dict]:
    """Blocks whose recorded `phoenix_status` disagrees with what their own content derives."""
    out: list[dict] = []
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            seq = rec.get("seq")
            if since is not None and isinstance(seq, int) and seq < since:
                continue
            recorded = rec.get("phoenix_status")
            wanted = derived_status(rec)
            if recorded != wanted:
                out.append({
                    "seq": seq,
                    "recorded": recorded,
                    "derived": wanted,
                    "severity": rec.get("severity"),
                    "event": rec.get("event"),
                    "source": rec.get("source"),
                    "ts": rec.get("ts"),
                    "hash": rec.get("hash"),
                })
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Blocks whose recorded status contradicts their own content")
    ap.add_argument("--ledger", type=Path, default=DEFAULT_LEDGER)
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    ap.add_argument("--all", action="store_true", help="print every contradiction, not just the first 10")
    ap.add_argument("--since", type=int, default=None, help="only blocks with seq >= N")
    args = ap.parse_args(argv)

    if not args.ledger.exists():
        print(f"ledger not found: {args.ledger}", file=sys.stderr)
        return 2

    hits = find_contradictions(args.ledger, args.since)
    if args.json:
        json.dump({"count": len(hits), "contradictions": hits}, sys.stdout, indent=1)
        sys.stdout.write("\n")
        return 0

    print(f"ledger: {args.ledger}")
    print(f"contradicting blocks: {len(hits)}")
    if hits:
        by_source: dict[str, int] = {}
        for h in hits:
            by_source[str(h["source"])] = by_source.get(str(h["source"]), 0) + 1
        print("by source:", dict(sorted(by_source.items(), key=lambda kv: -kv[1])))
        print("seq range:", hits[0]["seq"], "->", hits[-1]["seq"])
        print()
        shown = hits if args.all else hits[:10]
        for h in shown:
            print(f"  seq {h['seq']}  recorded {h['recorded']:<10} derived {h['derived']:<10} "
                  f"{h['severity']:<8} {str(h['event'])[:60]}")
        if not args.all and len(hits) > len(shown):
            print(f"  ... {len(hits) - len(shown)} more (use --all or --json)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
