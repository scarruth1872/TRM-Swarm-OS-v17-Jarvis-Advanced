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

"""538 + 540 — suite index and structured output validity."""

def main():
    s = get("/ie/suite")
    entries = s["suite"]
    print(f"suite {len(entries)} tasks | modules present {s['present_modules']}/10")
    for b in s["blocked"]:
        print(f"  BLOCKED task {b['task']}: {b['reason'][:90]}")

    probes = {
        "/ie/relations": {"text": "The kernel holds 219 modules."},
        "/ie/events": {"text": "Patients diagnosed with hypertension were treated with lisinopril."},
        "/ie/temporal": {"text": "Measured on 2026-11-25 after 3 weeks."},
        "/ie/negation": {"text": "No fever, but cough is present."},
        "/ie/uncertainty": {"text": "The result may indicate an infection."},
        "/ie/coref": {"text": "The kernel holds the registry. It enforces the guard."},
        "/ie/entities": {"text": "Staphylococcus aureus was screened."},
        "/ie/language": {"text": "The kernel holds the registry and enforces the guard on every build."},
        "/ie/structured": {"text": "The kernel holds 219 modules. It requires quorum evidence. Measured 2026-11-25."},
    }
    bad = []
    for p, body in probes.items():
        r = post(p, body)
        code = r.get("_error", 200)
        print(f"  {p:18} -> {code}")
        if code != 200: bad.append((p, code))

    # structured output: required keys present and every span resolves
    text = "The kernel holds 219 modules. It requires quorum evidence. Measured 2026-11-25 by Staphylococcus aureus assay."
    st = post("/ie/structured", {"text": text})
    required = ["schema", "sentences", "relations", "events", "temporal", "negations", "uncertainties", "coreferences", "entities"]
    missing = [k for k in required if k not in st]
    mismatched = 0
    for r in st.get("relations", []):
        for field in ("subject", "object"):
            sp = r[field]
            if bslice(text, sp["start"], sp["end"]) != sp["text"]: mismatched += 1
    for e in st.get("entities", []):
        m = e["mention"]
        if bslice(text, m["start"], m["end"]) != m["text"]: mismatched += 1
    print(f"schema keys missing {missing} | span mismatches {mismatched}")
    print(f"structured: {len(st.get('relations', []))} relations, {len(st.get('events', []))} events, "
          f"{len(st.get('temporal', []))} temporal, {len(st.get('entities', []))} entities")

    ok = len(entries) == 10 and not bad and not missing and mismatched == 0 and st["schema"] == "trm.ie.structured.v1"
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
