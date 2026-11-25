#!/usr/bin/env python3
"""Task 587 — scientific claim extraction: every extracted quantity must be the bytes it claims.

Mechanical precision check: for each extracted quantity the handler reports start/end into the sentence, and the
harness re-reads the file to confirm the literal sits exactly there. A quantity that cannot be located is a
fabrication, and the run fails.
"""
import json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(payload):
    req = urllib.request.Request("http://127.0.0.1:8022/reading/scientific", data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def main():
    files = sorted(ROOT.glob("docs/*.md"))[:8]
    total_claims = total_q = bad = 0
    for f in files:
        rel = str(f.relative_to(ROOT)).replace("\\", "/")
        raw = f.read_bytes()
        res = post({"path": rel, "limit": 200})
        bad += res["verbatim_failures"]
        total_claims += res["claims"]
        for c in res["detail"]:
            for q in c["quantities"]:
                total_q += 1
                got = raw[q["start"]:q["end"]].decode("utf-8", "replace")
                if got != q["text"]:
                    bad += 1
                    if bad <= 3:
                        print(f"  MISMATCH {rel} [{q['start']}:{q['end']}] expected {q['text']!r} got {got!r}")
    print(f"files {len(files)} | claims with quantities {total_claims} | quantities {total_q} | unlocatable {bad}")
    ok = total_claims > 0 and total_q > 0 and bad == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
