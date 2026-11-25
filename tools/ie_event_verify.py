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

def bslice(text, a, b):
    """Slice by BYTE offsets: the platform reports byte positions, Python str slicing is by codepoint, and the two
    diverge on any non-ASCII character (em dash, accents). Comparing with str slicing produced 54 phantom
    mismatches on real clinical text."""
    return text.encode("utf-8")[a:b].decode("utf-8", "replace")

def ct_sample():
    return json.loads((ROOT/"evidence"/"clinicaltrials_sample.json").read_text())

"""532 — event extraction on 200 real ClinicalTrials.gov records.

Checks: every trigger and argument span must resolve in the text it came from, and records that carry an
"Exclusion Criteria" section must produce criterion events — if the extractor never fires on real criteria text it
is not working on clinical notes.
"""

def main():
    sample = ct_sample()
    recs = sample["records"]
    checked = spans_bad = with_excl = excl_events = 0
    for rec in recs[:60]:
        text = " ".join(str(v) for k, v in rec.items() if isinstance(v, str) and k != "source_entry")
        r = post("/ie/events", {"text": text})
        if "_error" in r: print("ENDPOINT ERROR", r); return 1
        for e in r["detail"]:
            checked += 1
            if bslice(text, e["trigger"]["start"], e["trigger"]["end"]) != e["trigger"]["text"]:
                spans_bad += 1
            for a in e["arguments"]:
                if bslice(text, a["start"], a["end"]) != a["text"]:
                    spans_bad += 1
        if "exclusion criteria" in text.lower():
            with_excl += 1
            excl_events += r["by_kind"]["CRITERION_EXCLUSION"]
    print(f"records {len(recs[:60])} | events checked {checked} | span mismatches {spans_bad}")
    print(f"records with an exclusion section {with_excl} | CRITERION_EXCLUSION events {excl_events}")
    kinds = post("/ie/events", {"path": "docs/INFORMATION-EXTRACTION-RESEARCH.md"})["by_kind"]
    print(f"control document by_kind {kinds}")
    ok = spans_bad == 0 and checked > 0 and with_excl > 0 and excl_events > 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
