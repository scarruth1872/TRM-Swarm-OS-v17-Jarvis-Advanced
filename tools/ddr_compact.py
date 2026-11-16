#!/usr/bin/env python
"""Compact the DDR antibody store: collapse repeat firings of one rule into that rule.

Why this exists
---------------
`DigitalDNARepository.record_antibody` used to append unconditionally, and
`record_threat_antibody` appends a fresh CoT trace to every fix description. The shipped store
therefore reached 1,968 records carrying **7 distinct detection rules** -- 1,962 of them the same
`eval\\(|exec\\(|subprocess\\.call` rule on `*.py`. The headline "1,968 antibodies" was a row count
that grew with the number of times the injector fired, not with detection coverage.

What it does
------------
Groups records by (error_type, file_pattern, line_pattern), keeps the newest description for each
group, sums the firings into `duplicate_count`, and records `first_seen` / `all_sources`.

Nothing is deleted: every original record is written to
`swarm_v2_artifacts/DDR_ANTIBODIES.history.jsonl` with a `merged_into` field, and the pre-compaction
store is copied to `swarm_v2_artifacts/backups/` first. Per the errata rule records are re-measured,
never tidied away.

Usage
-----
    python tools/ddr_compact.py --dry-run     # report only
    python tools/ddr_compact.py               # backup, write history, compact
"""
import argparse
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STORE = ROOT / "swarm_v2_artifacts" / "DDR_ANTIBODIES.json"
HISTORY = ROOT / "swarm_v2_artifacts" / "DDR_ANTIBODIES.history.jsonl"
BACKUPS = ROOT / "swarm_v2_artifacts" / "backups"


def triple(r):
    return (r.get("error_type"), r.get("file_pattern"), r.get("line_pattern"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--store", default=str(STORE))
    args = ap.parse_args()

    store = Path(args.store)
    records = json.loads(store.read_text(encoding="utf-8"))
    print(f"store      : {store}")
    print(f"records    : {len(records)}")

    groups = {}
    for r in records:
        groups.setdefault(triple(r), []).append(r)
    print(f"distinct   : {len(groups)} rules")

    merged = []
    for key, rows in groups.items():
        rows.sort(key=lambda r: r.get("created_at") or "")
        newest = dict(rows[-1])
        newest["duplicate_count"] = len(rows)
        newest["first_seen"] = rows[0].get("created_at")
        newest["all_sources"] = sorted({r.get("recorded_by") or "system" for r in rows})
        # keep the strongest severity seen for the rule
        order = {"low": 0, "medium": 1, "high": 2, "critical": 3}
        newest["severity"] = max((r.get("severity") or "medium" for r in rows),
                                 key=lambda s: order.get(s, 1))
        merged.append(newest)

    merged.sort(key=lambda r: -(r.get("duplicate_count") or 0))
    print("\nrules after compaction:")
    for r in merged:
        print(f"  x{r['duplicate_count']:<5} {r.get('severity'):<9} {r.get('error_type')}")

    if args.dry_run:
        print("\ndry run: nothing written")
        return

    BACKUPS.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    backup = BACKUPS / f"DDR_ANTIBODIES.{stamp}.json"
    shutil.copy2(store, backup)
    print(f"\nbackup     : {backup}")

    keep_ids = {r.get("antibody_id") for r in merged}
    with HISTORY.open("a", encoding="utf-8") as fh:
        for r in records:
            row = dict(r)
            row["merged_into"] = next((m.get("antibody_id") for m in merged
                                       if triple(m) == triple(r)), None)
            row["compacted_at"] = stamp
            fh.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"history    : {HISTORY} (+{len(records)} rows)")

    store.write_text(json.dumps(merged, indent=2), encoding="utf-8")
    print(f"written    : {store} -> {len(merged)} records, "
          f"{sum(r['duplicate_count'] for r in merged)} total firings preserved")


if __name__ == "__main__":
    main()
