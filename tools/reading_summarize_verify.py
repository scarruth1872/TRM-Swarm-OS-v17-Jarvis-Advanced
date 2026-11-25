#!/usr/bin/env python3
"""Task 583 — multi-document summarisation by MMR: diversity must actually be lower, coverage must hold.

Acceptance as proposed to the council: at fixed length, MMR's mean pairwise redundancy must be lower than plain
top-k, and its coverage of the query's content words must not collapse.
"""
import json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

QUERIES = [
    "evidence ledger hash chain verification",
    "council quorum voting abstention",
    "druglikeness Lipinski violations compounds",
    "ternary K3 logic truth values",
    "GPU Vulkan inference throughput",
    "clinical trials extraction provenance",
    "reasoning engine forward chaining",
    "machine reading retrieval embeddings",
]

def main():
    lower = 0
    rows = []
    for q in QUERIES:
        r = post("/reading/summarize", {"query": q, "k": 5, "lambda": 0.6, "pool": 30})
        if r.get("verdict") == "NO_EVIDENCE":
            rows.append((q, None, None, None, None)); continue
        m = r["metrics"]
        div = m["mmr_redundancy"] < m["topk_redundancy"]
        lower += 1 if div else 0
        rows.append((q, m["mmr_redundancy"], m["topk_redundancy"], m["mmr_coverage"], m["topk_coverage"]))
    print(f"{'query':46} {'mmr_red':>8} {'topk_red':>9} {'mmr_cov':>8} {'topk_cov':>9}")
    for q, a, b, c, d in rows:
        if a is None: print(f"{q[:46]:46} {'NO EVIDENCE':>30}"); continue
        print(f"{q[:46]:46} {a:8.3f} {b:9.3f} {c:8.2f} {d:9.2f}")
    n = sum(1 for r in rows if r[1] is not None)
    print(f"\nMMR less redundant on {lower}/{n} queries")
    coverage_ok = all(r[3] is None or r[3] >= r[4] - 0.10 for r in rows)
    print(f"coverage preserved (within 10 points): {coverage_ok}")
    ok = n > 0 and lower >= max(1, int(0.6 * n)) and coverage_ok
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
