#!/usr/bin/env python3
"""Task 581 — verify the structural parse against the files themselves.

The check is mechanical: for every unit the kernel reports, the bytes it names must be the bytes on disk. A parser
that reports a plausible structure but wrong offsets fails, and that is the only kind of parser worth having.
"""
import json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def main():
    files = sorted(ROOT.glob("docs/*.md"))[:12]
    total_units = bad_bounds = bad_preview = 0
    kinds = {}
    for f in files:
        rel = str(f.relative_to(ROOT)).replace("\\", "/")
        raw = f.read_bytes()
        res = post("/reading/parse", {"path": rel})
        if res["bytes"] != len(raw):
            print(f"  SIZE MISMATCH {rel}: kernel {res['bytes']} vs disk {len(raw)}")
        for u in res["sections"]:
            total_units += 1
            kinds[u["kind"]] = kinds.get(u["kind"], 0) + 1
            if not (0 <= u["start"] < u["end"] <= len(raw)):
                bad_bounds += 1
                continue
            try:
                slice_text = raw[u["start"]:u["end"]].decode("utf-8", errors="replace")
            except Exception:
                bad_bounds += 1
                continue
            if slice_text[:120] != u["text_preview"]:
                bad_preview += 1
                if bad_preview <= 3:
                    print(f"  PREVIEW MISMATCH {rel} {u['kind']} [{u['start']}:{u['end']}]")
    print(f"files {len(files)} | units {total_units} | kinds {kinds}")
    print(f"bad bounds {bad_bounds} | preview mismatches {bad_preview}")
    ok = total_units > 0 and bad_bounds == 0 and bad_preview == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
