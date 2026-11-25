#!/usr/bin/env python3
"""Task 584 — every evidence span must resolve to identical bytes and a matching file hash.

No labels here either: the file is the ground truth. `unresolved` must be 0, each span's text must equal the
bytes it points at, and the sha256 must equal hashlib's.
"""
import hashlib, json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def main():
    files = sorted(ROOT.glob("docs/*.md"))[:10]
    spans = mismatched = hash_bad = unresolved = 0
    for f in files:
        rel = str(f.relative_to(ROOT)).replace("\\", "/")
        raw = f.read_bytes()
        res = post("/reading/evidence", {"path": rel, "limit": 300})
        want = hashlib.sha256(raw).hexdigest()
        if res["sha256"] != want:
            hash_bad += 1
            print(f"  HASH MISMATCH {rel}")
        for s in res["spans"]:
            spans += 1
            got = raw[s["start"]:s["end"]].decode("utf-8", errors="replace")
            if got != s["text"]:
                mismatched += 1
                if mismatched <= 3:
                    print(f"  SPAN MISMATCH {rel} [{s['start']}:{s['end']}]")
        unresolved += res["unresolved"]
    print(f"files {len(files)} | spans {spans} | unresolved {unresolved} | span mismatches {mismatched} | hash mismatches {hash_bad}")
    ok = spans > 0 and unresolved == 0 and mismatched == 0 and hash_bad == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
