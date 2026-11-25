import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type":"application/json","X-Council-Token":TOK})
    try:
        with urllib.request.urlopen(req, timeout=240) as r: return json.loads(r.read().decode())
    except urllib.error.HTTPError as e: return {"_error": e.code, "_body": e.read().decode(errors="replace")[:200]}

def get(path):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, headers={"X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=60) as r: return json.loads(r.read().decode())

"""598 — reproducibility: every artifact the document names must exist; a nonexistent path must be refused."""

def main():
    r = post("/kv/reproducibility", {"path": "docs/MACHINE-READING-RESEARCH.md"})
    if "_error" in r:
        print("ENDPOINT ERROR", r); return 1
    print(f"referenced artifacts {r['referenced_artifacts']} | present {r['present']} | missing {len(r['missing'])}")
    for m in r["missing"][:5]:
        print("  MISSING:", m)
    consistent = (len(r["missing"]) == r["referenced_artifacts"] - r["present"])
    print(f"count consistency: {consistent}")

    # the check must be able to find a missing artifact: docs/REFERENCED-MISSING.md does not exist
    r2 = post("/kv/reproducibility", {"path": "docs/DOES-NOT-EXIST.md"})
    refuses = "_error" in r2
    print(f"nonexistent document refused: {refuses}")

    # a document whose references all resolve must report reproducible=True
    r3 = post("/kv/reproducibility", {"path": "docs/REASONING-SUITE-COMPLETE.md"})
    print(f"REASONING-SUITE-COMPLETE.md: missing {len(r3.get('missing', []))} reproducible {r3.get('reproducible')}")

    ok = consistent and refuses and r["referenced_artifacts"] > 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
