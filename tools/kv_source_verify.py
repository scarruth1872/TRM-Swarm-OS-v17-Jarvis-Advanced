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

"""593 — source verification: the hash check must agree with hashlib, and an unrecorded source must NOT verify."""
import hashlib

def main():
    files = ["docs/MACHINE-READING-RESEARCH.md", "data/corpus_ledger_measured.json", "tools/reading_document_verify.py"]
    agree = 0
    for f in files:
        actual = hashlib.sha256((ROOT/f).read_bytes()).hexdigest()
        r = post("/kv/source", {"path": f, "recorded_sha": actual})
        if r.get("hash_matches") and r["actual_sha"] == actual:
            agree += 1
        else:
            print(f"  MISMATCH {f}: {r}")
    print(f"hash agreement with hashlib: {agree}/{len(files)}")

    # a wrong recorded hash must FAIL
    actual = hashlib.sha256((ROOT/files[0]).read_bytes()).hexdigest()
    bad = post("/kv/source", {"path": files[0], "recorded_sha": "0"*64})
    fails_correctly = bad.get("hash_matches") is False and "MISMATCH" in bad.get("detail", "")
    print(f"wrong recorded hash rejected: {fails_correctly}")

    # an unrecorded source must NOT verify
    unrec = post("/kv/source", {"path": files[1]})
    unrecorded_ok = unrec.get("hash_matches") is False and "NO RECORDED HASH" in unrec.get("detail", "")
    print(f"unrecorded source refused   : {unrecorded_ok}")

    # a path outside the repo must be refused
    outside = post("/kv/source", {"path": "../../../../Windows/win.ini"})
    outside_refused = "_error" in outside
    print(f"out-of-tree path refused    : {outside_refused}")

    ok = agree == len(files) and fails_correctly and unrecorded_ok and outside_refused
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
