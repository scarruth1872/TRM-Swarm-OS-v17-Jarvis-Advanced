#!/usr/bin/env python3
"""Task 590 — knowledge synthesis: every sentence must carry an evidence id that resolves.

The endpoint emits only extractive spans, so this is a mechanical check: for each sentence, the file at the cited
offset must contain those exact bytes and the sha256 must match. An untagged or unresolvable sentence fails.
"""
import hashlib, json, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(payload):
    req = urllib.request.Request("http://127.0.0.1:8022/reading/synthesis", data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=180) as r: return json.loads(r.read().decode())

def main():
    topics = ["council quorum evidence", "reasoning engine derivation", "retrieval index embeddings"]
    total = bad = untagged = 0
    for t in topics:
        r = post({"topic": t, "k": 5, "per_source": 2})
        if r.get("verdict") == "NO_EVIDENCE":
            print(f"  {t}: NO_EVIDENCE"); continue
        # skipped spans are reported, not counted as failures: the endpoint refuses to cite what it cannot
        # locate, which is the behaviour we want. Only a wrong citation or an untagged sentence is a failure.
        if r["unresolved_spans"]:
            print(f"  {t}: {r['unresolved_spans']} span(s) skipped as unlocatable (reported, not cited)")
        for sec in r["sections"]:
            for c in sec["claims"]:
                total += 1
                if not c.get("evidence_id") or c.get("tag") != "EVIDENCE":
                    untagged += 1
                f = ROOT / c["file"]
                raw = f.read_bytes()
                got = raw[c["offset"]:c["end"]].decode("utf-8", "replace")
                if got != c["claim"]:
                    bad += 1
                elif hashlib.sha256(raw).hexdigest() != c["sha256"]:
                    bad += 1
    print(f"sentences checked {total} | mismatched citations {bad} | untagged {untagged}")
    ok = total > 0 and bad == 0 and untagged == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
