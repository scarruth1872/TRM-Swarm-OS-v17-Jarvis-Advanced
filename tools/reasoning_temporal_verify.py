#!/usr/bin/env python3
"""Verify the kernel's temporal reasoning (task 577) against real ledger intervals.

Data: data/council_ledger.json — 1,586 real proposals with unix timestamps. Consecutive timestamps define the
window during which each proposal was the latest, giving ~1.5k real, non-overlapping-but-adjacent intervals.
Derived windows, real arithmetic: the intervals are constructed, the timings are measured.

Two independent checks:
  1. PAIR — the kernel's relation for sampled pairs must equal an independent Python implementation.
  2. TRIPLE — for sampled triples, relation(A,C) must be in compose(relation(A,B), relation(B,C)). This tests
     the composition algebra against the data itself: a table that admits too few relations fails here.
"""
import json, random, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def relation_py(a, b):
    """Independent implementation: half-open [start, end)."""
    (s1,e1),(s2,e2) = a,b
    if e1 < s2: return "BEFORE"
    if e1 == s2: return "MEETS"
    if e2 < s1: return "AFTER"
    if e2 == s1: return "MET_BY"
    if s1==s2 and e1==e2: return "EQUALS"
    if s1==s2: return "STARTS" if e1<e2 else "STARTED_BY"
    if e1==e2: return "FINISHES" if s1>s2 else "FINISHED_BY"
    if s1>s2 and e1<e2: return "DURING"
    if s1<s2 and e1>e2: return "CONTAINS"
    return "OVERLAPS" if s1<s2 else "OVERLAPPED_BY"

def main():
    led = json.loads((ROOT/"data"/"council_ledger.json").read_text())
    ts = sorted({r["timestamp"] for r in led if isinstance(r.get("timestamp"), int) and r["timestamp"] > 0})
    windows = [[ts[i], ts[i+1]] for i in range(len(ts)-1) if ts[i+1] > ts[i]]
    print(f"ledger entries {len(led)} | distinct timestamps {len(ts)} | windows {len(windows)}")
    if len(windows) < 20:
        print("not enough distinct timestamps to form intervals"); return 1

    rng = random.Random(20261125)

    # 1. pairs
    pairs = [rng.sample(range(len(windows)), 2) for _ in range(400)]
    payload = [[windows[i], windows[j]] for i, j in pairs]
    res = post("/reason/temporal", {"mode": "relation", "pairs": payload})
    got = [r["relation"] for r in res["relations"]]
    exp = [relation_py(windows[i], windows[j]) for i, j in pairs]
    bad = [(p, g, e) for p, g, e in zip(payload, got, exp) if g != e]
    print(f"pair check : {len(pairs)} pairs | agree {len(pairs)-len(bad)} | disagree {len(bad)}")
    for b in bad[:3]: print("   MISMATCH", b)

    # 2. triples (soundness of compose against real data)
    triples = []
    for _ in range(300):
        i, j, k = rng.sample(range(len(windows)), 3)
        triples.append([windows[i], windows[j], windows[k]])
    res2 = post("/reason/temporal", {"mode": "verify", "triples": triples})
    print(f"triple check: {res2['checked']} triples | sound {res2['ok']} | violations {res2['failures']}")
    for d in (res2.get("detail") or [])[:3]: print("   VIOLATION", d)

    ok = not bad and res2["failures"] == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
