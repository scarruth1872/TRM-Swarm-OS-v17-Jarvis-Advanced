#!/usr/bin/env python3
"""Regenerate phoenix_event_stream_audit_manifest.json from the live ledger.

The manifest is a frozen prefix attestation. It is NOT auto-updated, by design:
refreshing it is a deliberate act so the attested hash cannot drift silently.
Run this only when you intend to re-attest, then commit the manifest.

    python scripts/regen_phoenix_manifest.py [--dry-run]
"""
import json
import os
import sys
from datetime import datetime, timezone

W = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LEDGER = os.path.join(W, "swarm_v2_global_memory", "system_event_ledger.jsonl")
MANIFEST = os.path.join(W, "phoenix_event_stream_audit_manifest.json")


def main():
    dry = "--dry-run" in sys.argv
    with open(LEDGER, "r", encoding="utf-8") as f:
        lines = [l for l in f if l.strip()]
    first = json.loads(lines[0])
    last = json.loads(lines[-1])
    prev, breaks = first["prev_hash"], 0
    for i, line in enumerate(lines):
        e = json.loads(line)
        if e["seq"] != i + 1 or e["prev_hash"] != prev:
            breaks += 1
        prev = e["hash"]
    manifest = json.load(open(MANIFEST, "r", encoding="utf-8"))
    st = manifest["event_streams"]["system_event_ledger"]
    st["total_events"] = len(lines)
    st["final_chain_hash"] = last["hash"]
    st["attested_at_utc"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    manifest["summary"]["total_legacy_events_audited"] = len(lines)
    print("ledger events %d  chain breaks %d  final %s" % (len(lines), breaks, last["hash"]))
    if dry:
        print("dry run: manifest not written")
        return 1 if breaks else 0
    json.dump(manifest, open(MANIFEST, "w", encoding="utf-8"), indent=2)
    print("manifest rewritten:", MANIFEST)
    return 1 if breaks else 0


if __name__ == "__main__":
    sys.exit(main())
