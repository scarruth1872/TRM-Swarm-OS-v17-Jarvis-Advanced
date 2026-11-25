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

"""596 + 600 — the suite index and the comprehensive report."""

def main():
    s = get("/kv/suite")
    entries = s["suite"]
    missing = [e for e in entries if not e["module_present"]]
    print(f"suite entries {len(entries)} | modules present {s['present_modules']}/10")
    for m in missing: print("  MISSING MODULE:", m["id"], m["module"])

    probes = {
        "/kv/factcheck": {"claim": "The retrieval index holds 2,403 chunks.", "k": 4},
        "/kv/consistency": {"paths": ["docs/MACHINE-READING-RESEARCH.md"], "max_pairs": 5000},
        "/kv/source": {"path": "docs/MACHINE-READING-RESEARCH.md"},
        "/kv/conflict": {"a": {"label": "x", "measured": True, "kind": "PRIMARY"},
                         "b": {"label": "y", "measured": False, "kind": "SECONDARY"}},
        "/kv/uncertainty": {"claim": "The retrieval index holds 2,403 chunks.", "k": 4},
        "/kv/peerreview": {"path": "docs/MACHINE-READING-RESEARCH.md"},
        "/kv/reproducibility": {"path": "docs/MACHINE-READING-RESEARCH.md"},
        "/kv/retraction": {"path": "docs/MACHINE-READING-RESEARCH.md"},
        "/kv/report": {"paths": ["docs/MACHINE-READING-RESEARCH.md", "docs/REASONING-SUITE-COMPLETE.md"]},
    }
    bad = []
    for path, body in probes.items():
        r = post(path, body)
        code = r.get("_error", 200)
        print(f"  {path:24} -> {code}")
        if code != 200: bad.append((path, code))

    rep = post("/kv/report", {"paths": ["docs/MACHINE-READING-RESEARCH.md"]})
    print(f"report verdict {rep.get('verdict')} totals {rep.get('totals')}")

    missing_h = [e["harness"] for e in entries if not (ROOT / e["harness"]).exists()]
    for h in missing_h: print("  MISSING HARNESS:", h)

    ok = len(entries) == 10 and not missing and not bad and not missing_h and rep.get("verdict") in ("CLEAN", "FINDINGS")
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
