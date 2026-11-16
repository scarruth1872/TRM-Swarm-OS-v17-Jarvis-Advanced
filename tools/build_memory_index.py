#!/usr/bin/env python3
"""Build a flat id -> memory index from the TurboVec memory archive.

Why this exists: `swarm_v2_global_memory/turbo_memory.jsonl.gz` holds 168,429 memory
records, but the vector fabric (`fabric/fabric_vecs.npy`) only carries 42,805 ids and no
text. A vector hit is useless unless the id can be resolved back to the record it came
from, and gzip is not something the Rust kernel can read without a decompression crate.
So this tool writes a flat, line-delimited index the kernel can scan directly.

Outputs (both under data/):
  turbo_memory_index.jsonl  - one JSON per line: id, agent, ts, content (<=240 chars)
  turbo_memory_index_meta.json - counts, coverage against the fabric, source mtimes

Fail-closed rule: if the archive is missing or empty, write nothing and exit 2. A stale
index that looks authoritative is worse than no index.
"""
from __future__ import annotations

import argparse
import gzip
import json
import os
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GMEM = os.path.join(ROOT, "swarm_v2_global_memory")
ARCHIVE = os.path.join(GMEM, "turbo_memory.jsonl.gz")
LIVE = os.path.join(GMEM, "turbo_memory.jsonl")
IDS = os.path.join(GMEM, "fabric", "fabric_ids.json")
OUT_DIR = os.path.join(ROOT, "data")
OUT = os.path.join(OUT_DIR, "turbo_memory_index.jsonl")
META = os.path.join(OUT_DIR, "turbo_memory_index_meta.json")
CONTENT_CHARS = 240


def _open(path: str):
    return gzip.open(path, "rt", encoding="utf-8", errors="ignore") if path.endswith(".gz") \
        else open(path, "r", encoding="utf-8", errors="ignore")


def _agent_label(agent) -> str:
    # Some records carry an author dict rather than a name; keep it readable and short.
    if isinstance(agent, dict):
        return str(agent.get("author") or agent.get("name") or "unknown")[:32]
    return str(agent or "unknown")[:32]


def build(sources: list[str]) -> dict:
    seen: dict[str, dict] = {}
    read = 0
    bad = 0
    for path in sources:
        if not os.path.exists(path):
            continue
        with _open(path) as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                read += 1
                try:
                    d = json.loads(line)
                except Exception:
                    bad += 1
                    continue
                mid = d.get("id")
                if not mid:
                    continue
                content = d.get("content")
                if content is None:
                    # ledger-style records carry their text elsewhere
                    data = d.get("data") if isinstance(d.get("data"), dict) else {}
                    content = d.get("event") or data.get("task") or json.dumps(d)[:CONTENT_CHARS]
                seen[str(mid)] = {
                    "id": str(mid),
                    "agent": _agent_label(d.get("agent")),
                    "ts": str(d.get("timestamp") or d.get("ts") or "")[:32],
                    "content": str(content)[:CONTENT_CHARS].replace("\n", " "),
                }
    if not seen:
        return {"error": "no records", "read": read, "bad": bad}

    os.makedirs(OUT_DIR, exist_ok=True)
    tmp = OUT + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        for rec in seen.values():
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    os.replace(tmp, OUT)

    fabric_ids = set()
    if os.path.exists(IDS):
        with open(IDS, encoding="utf-8") as fh:
            fabric_ids = set(json.load(fh))
    indexed = set(seen)
    agents: dict[str, int] = {}
    for rec in seen.values():
        agents[rec["agent"]] = agents.get(rec["agent"], 0) + 1
    meta = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "sources": [os.path.relpath(s, ROOT) for s in sources if os.path.exists(s)],
        "records_read": read,
        "records_indexed": len(seen),
        "unparseable": bad,
        "content_chars": CONTENT_CHARS,
        "index_bytes": os.path.getsize(OUT),
        "agents": dict(sorted(agents.items(), key=lambda kv: -kv[1])[:20]),
        "fabric_ids": len(fabric_ids),
        # G162: the denominator was len(fabric_ids), so this could only ever print 100% - it measured
        # "are fabric ids in the index", not "how much of the archive is embedded". Publish both, and
        # divide archive coverage by the archive.
        "fabric_coverage_pct": round(100.0 * len(fabric_ids & indexed) / len(indexed), 2) if indexed else None,
        "fabric_coverage_note": "share of indexed archive records that have a vector (denominator = records_indexed)",
        "fabric_index_ratio_pct": round(100.0 * len(fabric_ids & indexed) / len(fabric_ids), 2) if fabric_ids else None,
        "fabrics_unresolvable": len(fabric_ids - indexed),
        "indexed_not_in_fabric": len(indexed - fabric_ids),
    }
    with open(META, "w", encoding="utf-8") as fh:
        json.dump(meta, fh, indent=2)
    return meta


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--include-live", action="store_true",
                    help="also merge turbo_memory.jsonl (the un-compacted live file)")
    ap.add_argument("--check", action="store_true", help="exit 1 if the index is missing or older than its sources")
    args = ap.parse_args()

    sources = [ARCHIVE]
    if args.include_live:
        sources.append(LIVE)

    newest_src = max((os.path.getmtime(p) for p in sources if os.path.exists(p)), default=0)
    if args.check:
        if not os.path.exists(OUT) or not os.path.exists(META):
            print("STALE: index missing", file=sys.stderr)
            return 1
        if os.path.getmtime(OUT) < newest_src:
            print("STALE: index older than source archive", file=sys.stderr)
            return 1
        print("CURRENT")
        return 0

    if not any(os.path.exists(p) for p in sources):
        print(f"NO SOURCE: neither {ARCHIVE} nor {LIVE} exists", file=sys.stderr)
        return 2
    meta = build(sources)
    if "error" in meta:
        print(f"FAILED: {meta}", file=sys.stderr)
        return 2
    print(json.dumps(meta, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
