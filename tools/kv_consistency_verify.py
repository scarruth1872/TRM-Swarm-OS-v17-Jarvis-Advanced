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

"""592 — knowledge consistency: cross-document contradictions and the ledger gaps.

The ledger gaps are checked against an independent recomputation from the JSON, so the endpoint cannot report a
gap list that disagrees with the file it read.
"""

def main():
    paths = ["docs/MACHINE-READING-RESEARCH.md", "docs/REASONING-SUITE-COMPLETE.md", "docs/REASONING-ABSENCES-RESEARCH.md"]
    r = post("/kv/consistency", {"paths": paths, "max_pairs": 200000})
    if "_error" in r:
        print("ENDPOINT ERROR", r); return 1
    print(f"files {r['files']} | sentences {r['sentences']} | pairs compared {r['pairs_compared']}")
    print(f"cross-document contradictions: {r['contradictions']}")
    for d in r["detail"][:3]:
        print(f"  {d['kind']}: {d['detail'][:90]}")

    ledger = json.loads((ROOT/"data"/"corpus_ledger_measured.json").read_text())
    expected = []
    for name, rec in (ledger.get("shards") or {}).items():
        c, m = rec.get("claimed"), rec.get("measured")
        if c is not None and m is not None and c != m:
            expected.append(name)
    got = [g["shard"] for g in r["ledger_gaps"]]
    print(f"ledger gaps: endpoint {len(got)} | recomputed {len(expected)} | agree: {sorted(got) == sorted(expected)}")
    ok = sorted(got) == sorted(expected) and r["sentences"] > 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
