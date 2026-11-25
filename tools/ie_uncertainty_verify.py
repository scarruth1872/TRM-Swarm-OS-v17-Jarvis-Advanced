import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type":"application/json","X-Council-Token":TOK})
    try:
        with urllib.request.urlopen(req, timeout=180) as r: return json.loads(r.read().decode())
    except urllib.error.HTTPError as e: return {"_error": e.code, "_body": e.read().decode(errors="replace")[:220]}

def get(path):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, headers={"X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=60) as r: return json.loads(r.read().decode())

def ct_sample():
    return json.loads((ROOT/"evidence"/"clinicaltrials_sample.json").read_text())

"""537 — uncertainty: hedged sentences must be flagged, plain assertions must not."""

HEDGED = [
    "The result may indicate an infection.",
    "This possibly explains the discrepancy.",
    "The mechanism is unknown.",
    "It cannot be excluded that the effect is noise.",
    "The value is approximately 2403.",
    "The outcome appears consistent with the model.",
]
PLAIN = [
    "The kernel holds 219 modules.",
    "The council enforced the veto at 14 approvals.",
    "The index contains 2403 chunks from 191 files.",
    "The guard raised on both trees.",
    "The harness compared the two implementations.",
]

def main():
    hits = missed = 0
    for s in HEDGED:
        r = post("/ie/uncertainty", {"text": s})
        if r["hedged_sentences"] >= 1: hits += 1
        else: missed += 1; print("  MISSED:", s)
    fp = 0
    for s in PLAIN:
        r = post("/ie/uncertainty", {"text": s})
        if r["hedged_sentences"] > 0:
            fp += 1; print("  FALSE POSITIVE:", s, "->", r["detail"][0]["cue"])
    print(f"hedged caught {hits}/{len(HEDGED)} | false positives {fp}/{len(PLAIN)}")
    sample = ct_sample()
    flagged = 0
    for rec in sample["records"][:50]:
        text = rec.get("brief_summary") or ""
        if text:
            r = post("/ie/uncertainty", {"text": text})
            flagged += r["hedged_sentences"]
    print(f"real summaries: {flagged} hedged sentences flagged across 50 records")
    ok = hits >= 5 and fp <= 1
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
