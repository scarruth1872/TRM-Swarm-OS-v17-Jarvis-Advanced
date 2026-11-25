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

"""597 — peer review: the check must FIRE on a constructed line and stay quiet on a clean one."""

DIRTY = """We measured 406 compounds at R2 0.72 and it works.
TODO: finish this section.
The retrieval index has 2403 chunks and 191 files and 768 dimensions.
"""
CLEAN = """Measured in tools/reasoning_verify.py: 406/406 agree, offsets at file[0:120].
See docs/MACHINE-READING-RESEARCH.md for the retrieval numbers.
"""

def main():
    d = post("/kv/peerreview", {"text": DIRTY})
    c = post("/kv/peerreview", {"text": CLEAN})
    print(f"dirty: blocking {d['blocking']} warnings {d['warnings']} notes {d['notes']} passes {d['passes']}")
    print(f"clean: blocking {c['blocking']} warnings {c['warnings']} notes {c['notes']} passes {c['passes']}")
    fires = d["blocking"] >= 1 and any(f["rule"] == "assertion-without-provenance" for f in d["findings"])
    quiet = c["blocking"] == 0
    placeholder = any(f["rule"] == "placeholder" for f in d["findings"])
    print(f"blocking rule fires: {fires} | placeholder flagged: {placeholder} | clean doc passes: {quiet}")

    # a real document, reported not judged
    real = post("/kv/peerreview", {"path": "docs/MACHINE-READING-RESEARCH.md"})
    print(f"real doc: blocking {real['blocking']} warnings {real['warnings']} notes {real['notes']}")

    ok = fires and quiet and placeholder
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
