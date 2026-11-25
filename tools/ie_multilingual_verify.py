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

"""539 — the language gate: English passes, other scripts are REFUSED with a reason."""

CASES = [
    ("The kernel holds the registry and enforces the guard on every build.", True, None),
    ("内核保持注册表并在每次构建时强制执行守卫。", False, "CJK"),
    ("Ядро содержит реестр и обеспечивает защиту при каждой сборке.", False, "Cyrillic"),
    ("نواة تحتفظ بالسجل وتفرض الحماية في كل بناء.", False, "Arabic"),
]

def main():
    ok = 0
    for text, should_pass, script in CASES:
        r = post("/ie/language", {"text": text})
        if "_error" in r: print("ENDPOINT ERROR", r); return 1
        allowed = r["extraction_allowed"]
        good = allowed == should_pass
        if not should_pass:
            good = good and script in (r.get("reason") or "")
        ok += 1 if good else 0
        print(f"  {text[:26]!r:30} allowed={allowed} -> {'ok' if good else 'WRONG'}")

    # and extraction must actually be refused, not merely reported
    refused = post("/ie/structured", {"text": "内核保持注册表并在每次构建时强制执行守卫。"})
    refused_properly = "_error" in refused
    print(f"structured extraction on CJK refused: {refused_properly}")
    ok_all = ok == len(CASES) and refused_properly
    print("RESULT:", "PASS" if ok_all else "FAIL")
    return 0 if ok_all else 1

if __name__ == "__main__":
    sys.exit(main())
