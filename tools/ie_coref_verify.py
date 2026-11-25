import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_bytes().decode("utf-8").strip()
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
    return json.loads((ROOT/"evidence"/"clinicaltrials_sample.json").read_bytes().decode("utf-8"))

"""534 — coreference: constructed paragraphs with known antecedents."""

CASES = [
    ("The kernel holds the registry. It enforces the guard on every build.", "kernel"),
    ("The council reviewed the proposal. It reached no consensus.", "council"),
    ("The harness measures calibration. It reports a Brier score.", "harness"),
]

def main():
    ok = 0
    for para, expected in CASES:
        r = post("/ie/coref", {"text": para})
        ants = [c["antecedent"]["text"].lower() for c in r["detail"]]
        good = expected in ants
        ok += 1 if good else 0
        print(f"  expected {expected!r:10} | antecedents {ants} -> {'ok' if good else 'WRONG'}")
    # spans must resolve
    para = CASES[0][0]
    r = post("/ie/coref", {"text": para})
    bad = 0
    for c in r["detail"]:
        a = c["antecedent"]
        if bslice(para, a["start"], a["end"]) != a["text"]: bad += 1
        for m in c["mentions"]:
            if bslice(para, m["start"], m["end"]) != m["text"]: bad += 1
    print(f"cases {len(CASES)} | correct {ok} | span mismatches {bad}")
    ok_all = ok >= 2 and bad == 0
    print("RESULT:", "PASS" if ok_all else "FAIL")
    return 0 if ok_all else 1

if __name__ == "__main__":
    sys.exit(main())
