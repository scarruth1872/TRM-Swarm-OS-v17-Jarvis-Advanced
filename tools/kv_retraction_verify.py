#!/usr/bin/env python3
"""599 — retraction detection.

Three checks:
  1. the detector FIRES on a constructed ERRATUM (a check never shown to fire is not a check);
  2. it finds the real corrections that exist in the corpus, and applies one only to the claim it names;
  3. it reports how many documents in docs/ carry a marker — a measured absence, not an assumption.
"""
import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type":"application/json","X-Council-Token":TOK})
    try:
        with urllib.request.urlopen(req, timeout=180) as r: return json.loads(r.read().decode())
    except urllib.error.HTTPError as e: return {"_error": e.code, "_body": e.read().decode(errors="replace")[:200]}

CONSTRUCTED = """# Doc

## ERRATUM
The earlier claim of 97 phantom modules was wrong; all 97 exist in the cdylib tree.

More prose here.
"""

def main():
    c = post("/kv/retraction", {"text": CONSTRUCTED, "claim": "97 phantom modules did not exist"})
    fires = c["retractions"] >= 1 and c.get("claim_retracted") is True
    print(f"constructed ERRATUM found: {c['retractions']} | applies to named claim: {c.get('claim_retracted')}")

    stray = post("/kv/retraction", {"text": CONSTRUCTED, "claim": "the GPU sustains 36 tokens per second"})
    not_applied = stray.get("claim_retracted") is False
    print(f"unrelated claim NOT covered : {not_applied}")

    real = post("/kv/retraction", {"path": "docs/CHECKPOINT-20261123.md",
                                   "claim": "the TRM-ABX lead IDs exist only in the board"})
    if "_error" in real:
        print("ENDPOINT ERROR", real); return 1
    print(f"CHECKPOINT-20261123.md: {real['retractions']} correction(s)")
    for d in real["detail"][:2]:
        print(f"  line {d['line']} [{d['marker']}]: {d['body'][:110]}")
    real_found = real["retractions"] >= 1
    real_applied = real.get("claim_retracted") is True
    print(f"real correction found: {real_found} | applies to the named claim: {real_applied}")

    # measured absence across the corpus
    scanned = marked = 0
    for f in sorted(ROOT.glob("docs/*.md")):
        scanned += 1
        r = post("/kv/retraction", {"path": str(f.relative_to(ROOT)).replace("\\", "/")})
        if r.get("retractions", 0) > 0:
            marked += 1
    print(f"corpus scan: {scanned} documents, {marked} carry a correction marker")

    ok = fires and not_applied and real_found and real_applied
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
