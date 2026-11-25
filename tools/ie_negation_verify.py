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

"""536 — negation against the curator's own labels.

Ground truth comes from the data, not from me: criteria under "Exclusion Criteria" are exclusions by the submitter's
own label. Two things are measured separately:
  * PRECISION side — inclusion-section sentences must rarely be flagged negated by a cue;
  * RECALL side — how many exclusion-section sentences a CUE catches (the rest are negative only by section).
Reporting both is the point: a detector that flags everything would score 100% on recall and 0% on precision.
"""
import re

def main():
    sample = ct_sample()
    inc_total = inc_neg = exc_total = exc_neg = exc_section = 0
    for rec in sample["records"][:80]:
        text = rec.get("eligibility_criteria") or ""
        if "exclusion criteria" not in text.lower():
            continue
        r = post("/ie/negation", {"text": text})
        if "_error" in r: print("ENDPOINT ERROR", r); return 1
        for row in r["detail"]:
            sec = row.get("criteria_section")
            s = row["sentence"].strip()
            if len(s) < 25:
                continue
            if sec == "CRITERION_INCLUSION":
                inc_total += 1
                inc_neg += 1 if row["negated"] else 0
            elif sec == "CRITERION_EXCLUSION":
                exc_total += 1
                exc_neg += 1 if row["negated"] else 0
                exc_section += 1
    print(f"inclusion sentences {inc_total} | flagged negated {inc_neg} ({inc_neg/max(1,inc_total):.1%})  <- precision side")
    print(f"exclusion sentences {exc_total} | flagged negated by CUE {exc_neg} ({exc_neg/max(1,exc_total):.1%})  <- cue-only recall")
    print(f"exclusion sentences labelled negative by SECTION: {exc_section}/{exc_total} (100% by construction)")
    # a detector that fires on everything would show inclusion ≈ exclusion; require a clear separation
    separation = (inc_neg/max(1,inc_total)) < 0.35
    fires_on_exclusion = exc_neg > 0
    ok = inc_total > 20 and exc_total > 20 and separation and fires_on_exclusion
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
