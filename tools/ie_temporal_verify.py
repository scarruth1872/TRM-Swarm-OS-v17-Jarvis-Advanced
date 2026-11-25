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

"""535 — temporal extraction checked against an independent Python parse."""
import re
from datetime import date

ISO = re.compile(r"\b(\d{4})-(\d{2})-(\d{2})\b")

def main():
    docs = ["docs/INFORMATION-EXTRACTION-RESEARCH.md", "docs/KNOWLEDGE-VERIFICATION.md",
            "docs/REASONING-SUITE-COMPLETE.md", "docs/MACHINE-READING-RESEARCH.md"]
    iso_total = iso_match = 0
    for d in docs:
        text = (ROOT/d).read_text()
        expected = set()
        for m in ISO.finditer(text):
            y, mo, dd = int(m.group(1)), int(m.group(2)), int(m.group(3))
            try:
                date(y, mo, dd)
                expected.add(m.group(0))
            except ValueError:
                pass  # impossible date: the extractor must NOT produce it either
        r = post("/ie/temporal", {"path": d})
        if "_error" in r: print("ENDPOINT ERROR", r); return 1
        got = {t["normalized"] for t in r["detail"] if t["kind"] == "DATE"}
        iso_total += len(expected)
        missing = expected - got
        iso_match += len(expected) - len(missing)
        if missing:
            print(f"  {d}: MISSING {sorted(missing)[:3]}")
        extra_impossible = [t for t in r["detail"] if t["kind"] == "DATE" and t["normalized"] not in expected and ISO.fullmatch(t["normalized"])]
        if extra_impossible:
            print(f"  {d}: EXTRACTED IMPOSSIBLE {extra_impossible[:2]}")
    print(f"ISO dates: independent parse {iso_total} | extractor matched {iso_match}")
    ok = iso_total > 0 and iso_match == iso_total
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
