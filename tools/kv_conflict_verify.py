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

"""594 — conflict resolution on cases whose correct answer is known in advance.

Four cases, each isolating one rule plus the tie. Accuracy is the metric; a case that resolves by the wrong rule is
a failure even if it names the expected winner.
"""

CASES = [
    # (a, b, expected_winner, expected_rule)
    ({"label": "ledger", "measured": True, "kind": "SECONDARY", "date": "2026-11-01"},
     {"label": "summary", "measured": False, "kind": "PRIMARY", "date": "2026-11-20"},
     "ledger", "measured over claimed"),
    ({"label": "evidence", "measured": True, "kind": "PRIMARY", "date": "2026-11-01"},
     {"label": "doc", "measured": True, "kind": "SECONDARY", "date": "2026-11-20"},
     "evidence", "primary over secondary"),
    ({"label": "old", "measured": True, "kind": "PRIMARY", "date": "2026-11-01"},
     {"label": "new", "measured": True, "kind": "PRIMARY", "date": "2026-11-20"},
     "new", "newer over older"),
    ({"label": "a", "measured": True, "kind": "PRIMARY", "date": "2026-11-01"},
     {"label": "b", "measured": True, "kind": "PRIMARY", "date": "2026-11-01"},
     None, "UNRESOLVED"),
]

def main():
    ok = 0
    for a, b, winner, rule in CASES:
        r = post("/kv/conflict", {"a": a, "b": b})
        got_w, got_r = r.get("winner"), r.get("rule")
        good = got_w == winner and got_r == rule
        ok += 1 if good else 0
        print(f"  {a['label']} vs {b['label']}: winner={got_w} rule={got_r!r} -> {'ok' if good else 'WRONG (expected ' + str(winner) + '/' + rule + ')'}")
    print(f"cases {len(CASES)} | correct {ok}")
    print("RESULT:", "PASS" if ok == len(CASES) else "FAIL")
    return 0 if ok == len(CASES) else 1

if __name__ == "__main__":
    sys.exit(main())
