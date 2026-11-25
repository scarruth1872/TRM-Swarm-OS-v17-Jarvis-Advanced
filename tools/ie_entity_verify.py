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

"""533 — entity linking against the LOCAL vocabulary, and an explicit refusal to claim UMLS."""

def main():
    r = post("/ie/entities", {"text": "Staphylococcus aureus and Enterococcus faecium were screened."})
    if "_error" in r: print("ENDPOINT ERROR", r); return 1
    print(f"vocabulary {r['vocabulary']} | size {r['vocab_size']} | entities {r['entities']}")
    print("umls:", r["umls"][:80])
    ids = {e["id"] for e in r["detail"]}
    names = [e["mention"]["text"] for e in r["detail"]]
    print(f"linked {names} -> {sorted(ids)}")
    correct = ids == {"UP000008816", "UP000005269"}
    claims_umls = r["vocabulary"] != "local"
    print(f"ids correct: {correct} | claims UMLS: {claims_umls}")

    # a mention that is not in the vocabulary must not be linked
    none = post("/ie/entities", {"text": "Escherichia coli is not in this vocabulary."})
    no_false = none["entities"] == 0
    print(f"out-of-vocabulary entity not linked: {no_false} ({none['entities']} linked)")

    # spans resolve
    text = "Staphylococcus aureus and Enterococcus faecium were screened."
    bad = sum(1 for e in r["detail"] if bslice(text, e["mention"]["start"], e["mention"]["end"]) != e["mention"]["text"])
    print(f"span mismatches {bad}")
    ok = correct and (not claims_umls) and no_false and bad == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
