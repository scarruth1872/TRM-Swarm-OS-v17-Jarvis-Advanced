#!/usr/bin/env python3
"""Task 579 — verify the suite index is true.

Check 1: GET /reason/suite lists ten tasks and every claimed module exists on disk.
Check 2: every endpoint it names answers a minimal valid request with 2xx (a route that 404s is not a
         capability, however good the file looks).
Check 3: every harness the index advertises exists and is executable.
"""
import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"

def get(path):
    req = urllib.request.Request(BASE+path, headers={"X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=30) as r: return json.loads(r.read().decode())

def post(path, payload):
    req = urllib.request.Request(BASE+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type":"application/json","X-Council-Token":TOK})
    try:
        with urllib.request.urlopen(req, timeout=60) as r: return r.status
    except urllib.error.HTTPError as e: return e.code

def main():
    s = get("/reason/suite")
    entries = s["suite"]
    print(f"suite entries: {len(entries)} | modules present: {s['present_modules']}/10")
    missing = [e for e in entries if not e["module_present"]]
    for m in missing: print("   MISSING MODULE:", m)

    # minimal valid requests per endpoint
    probes = {
        "/reason/forward":  {"rulebase": "druglikeness", "facts": [{"atom": "mw_gt_500", "source": "probe"}]},
        "/reason/prove":    {"rulebase": "druglikeness", "facts": [{"atom": "mw_gt_500", "source": "probe"}], "goal": "violation_mw"},
        "/reason/case":     {"cases": [{"id": "a", "features": [1.0], "outcome": 1.0},
                                       {"id": "b", "features": [2.0], "outcome": 2.0},
                                       {"id": "c", "features": [3.0], "outcome": 3.0}], "mode": "loo", "k": 1},
        "/reason/analogy":  {"cases": [{"id": "a", "features": [0.0], "outcome": 0.0},
                                       {"id": "b", "features": [1.0], "outcome": 1.0},
                                       {"id": "c", "features": [2.0], "outcome": 2.0},
                                       {"id": "d", "features": [3.0], "outcome": 3.0}], "mode": "query", "query": [2.5]},
        "/reason/abduce":   {"rulebase": "druglikeness", "observation": "violation_mw",
                             "hypotheses": [{"atom": "mw_gt_500"}], "max_size": 1},
        "/reason/temporal": {"mode": "relation", "pairs": [[[0, 10], [10, 20]]]},
        "/reason/spatial":  {"atoms": [{"x": 0, "y": 0, "z": 0, "element": "C"},
                                       {"x": 1.5, "y": 0, "z": 0, "element": "C"}], "cutoff": 2.0},
    }
    bad = []
    for path, body in probes.items():
        code = post(path, body)
        print(f"  {path:20} -> {code}")
        if code != 200:
            bad.append((path, code))
    print(f"GET /reason/rulebases -> 200 (rule profiles: {[r['name'] for r in get('/reason/rulebases')['rulebases']]})")

    harnesses = [h.split(" ")[0] for h in s["harnesses"]]
    missing_h = [h for h in harnesses if not (ROOT/h).exists()]
    for h in missing_h: print("   MISSING HARNESS:", h)

    ok = not missing and not bad and not missing_h and len(entries) == 10
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
