#!/usr/bin/env python3
"""Task 588 — evidence-based recommendation, including the counterfactual.

Acceptance as proposed: flipping the evidence must flip the recommendation, and citation-free support must be
refused rather than answered. Both are tested here against the live endpoint.
"""
import json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()

def post(payload):
    req = urllib.request.Request("http://127.0.0.1:8022/reading/recommend", data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def main():
    # 1. REFUSAL: an option with no retrievable evidence must be refused, not ranked. A gibberish token set does
    #    the job without relying on lexical overlap heuristics.
    gibberish = {"id": "B", "query": "zzqx jjkw vbbn qqpz", "k": 4}
    strong = {"id": "A", "query": "council quorum distinct voter veto arithmetic"}
    r1 = post({"options": [strong, gibberish], "k": 4})
    refused_ids = (r1.get("unsupported") or []) + [e.get("id") for e in (r1.get("below_support") or [])]
    refused = r1.get("verdict") == "INSUFFICIENT_EVIDENCE" and "B" in refused_ids
    print(f"citation-free option refused : {refused} ({r1.get('verdict')}, unsupported={r1.get('unsupported')})")

    # 2. RANKING follows evidence: swapping the queries must swap the order
    a = {"id": "A", "query": "ternary K3 logic truth values"}
    b = {"id": "B", "query": "council quorum distinct voter veto arithmetic"}
    r2 = post({"options": [a, b], "k": 4})
    order1 = [x["id"] for x in r2.get("ranking", [])]
    r3 = post({"options": [{"id": "A", "query": b["query"]}, {"id": "B", "query": a["query"]}], "k": 4})
    order2 = [x["id"] for x in r3.get("ranking", [])]
    print(f"ranking as stated            : {order1}")
    print(f"ranking with queries swapped : {order2}")
    flipped = bool(order1) and bool(order2) and order1 != order2
    print(f"recommendation follows the evidence: {flipped}")

    # 3. every ranked option carries citations, and the thin ones say so
    cites_ok = all(len(x.get("citations") or []) > 0 for x in r2.get("ranking", []))
    support_reported = all("lexical_support" in x for x in r2.get("ranking", []))
    print(f"every ranked option carries citations: {cites_ok}")
    print(f"lexical support reported per option  : {support_reported}")
    for x in r2.get("ranking", []):
        print(f"   {x['id']}: weight {x['evidence_weight']:.3f} | support {x['lexical_support']:.2f} | {x['citations'][0]['source']}")

    ok = refused and flipped and cites_ok and support_reported and r2.get("verdict") == "RANKED"
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
