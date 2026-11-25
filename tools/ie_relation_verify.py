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

"""531 — relation extraction: injected triples must come back, and every span must round-trip."""

INJECT = [
    ("The kernel", "holds", "219 modules"),
    ("The ledger", "requires", "quorum evidence"),
    ("The council", "enforces", "veto arithmetic"),
    ("The index", "contains", "2403 chunks"),
    ("The gate", "blocks", "dangerous approvals"),
    ("The suite", "implements", "ten tasks"),
    ("The guard", "validates", "both trees"),
    ("The bridge", "reads", "live endpoints"),
    ("The harness", "measures", "calibration"),
    ("The parser", "defines", "byte offsets"),
]

def main():
    text = " ".join(f"{s} {v} {o}." for s, v, o in INJECT)
    r = post("/ie/relations", {"text": text})
    if "_error" in r:
        print("ENDPOINT ERROR", r); return 1
    got = {(x["subject"]["text"].lower().replace("the ", ""), x["predicate"], x["object"]["text"].lower())
           for x in r["detail"]}
    hits = 0
    for s, v, o in INJECT:
        key = (s.lower().replace("the ", ""), v, o.lower())
        if any(g[1] == v and g[0] in key[0] and o.split()[0] in g[2] for g in got):
            hits += 1
    print(f"injected triples {len(INJECT)} | recovered {hits} ({hits/len(INJECT):.1%})")

    # real document: every span must resolve
    real = post("/ie/relations", {"path": "docs/KNOWLEDGE-VERIFICATION.md"})
    raw = (ROOT/"docs/KNOWLEDGE-VERIFICATION.md").read_bytes().decode("utf-8")
    bad = sum(1 for x in real["detail"]
              if bslice(raw, x["subject"]["start"], x["subject"]["end"]) != x["subject"]["text"]
              or bslice(raw, x["object"]["start"], x["object"]["end"]) != x["object"]["text"])
    print(f"real doc: {real['relations']} relations | span mismatches {bad}")
    ok = hits >= 7 and bad == 0 and real["relations"] > 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
