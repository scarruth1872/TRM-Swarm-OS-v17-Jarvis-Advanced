#!/usr/bin/env python3
"""Task 586 — verify the machine reading suite index and that every endpoint it names answers."""
import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"

def get(p):
    req = urllib.request.Request(BASE+p, headers={"X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=30) as r: return json.loads(r.read().decode())

def post(p, payload):
    req = urllib.request.Request(BASE+p, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type":"application/json","X-Council-Token":TOK})
    try:
        with urllib.request.urlopen(req, timeout=90) as r: return r.status
    except urllib.error.HTTPError as e: return e.code

def main():
    s = get("/reading/suite")
    entries = s["suite"]
    missing = [e for e in entries if not e["module_present"]]
    print(f"entries {len(entries)} | modules present {s['present_modules']}/10")
    for m in missing: print("  MISSING MODULE:", m["id"], m["module"])

    probes = {
        "/reading/parse": {"path": "docs/MACHINE-READING-RESEARCH.md"},
        "/reading/evidence": {"path": "docs/MACHINE-READING-RESEARCH.md", "limit": 5},
        "/reading/qa": {"question": "how many chunks are in the retrieval index", "k": 3},
        "/reading/summarize": {"query": "retrieval index chunks", "k": 3},
        "/reading/claim": {"claim": "The retrieval index holds 2,403 chunks.", "k": 3},
        "/reading/scientific": {"path": "docs/MACHINE-READING-RESEARCH.md", "limit": 20},
        "/reading/contradictions": {"sentences": ["The index holds 2,403 chunks.", "The index holds 558 chunks."]},
        "/reading/recommend": {"options": [{"id": "A", "query": "retrieval index"}, {"id": "B", "query": "quorum veto"}], "k": 2},
        "/reading/synthesis": {"topic": "retrieval index chunks", "k": 2},
    }
    bad = []
    for path, body in probes.items():
        code = post(path, body)
        print(f"  {path:26} -> {code}")
        if code != 200: bad.append((path, code))

    missing_h = [e["harness"] for e in entries if not (ROOT / e["harness"]).exists()]
    for h in missing_h: print("  MISSING HARNESS:", h)

    ok = len(entries) == 10 and not missing and not bad and not missing_h
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
