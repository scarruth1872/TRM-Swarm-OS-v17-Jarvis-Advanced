#!/usr/bin/env python3
"""322 — fuzzing suite for every JSON parser on the request path.

A 500 from this server means the handler PANICKED (tiny_http turns an unwound handler into 500 with an empty body)
— and this platform has already been bitten by exactly that: a char-boundary panic made 21 of 40 documents
unreadable, and a byte-slice panic killed the temporal extractor. So the signal here is precise:

    5xx  -> panic in a parser/handler, a real defect
    4xx  -> correctly rejected
    200  -> accepted (checked for absurd values that should have been rejected)

Every endpoint gets the same payload classes, so a bug found once is looked for everywhere.
"""
import json, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"
OUT = ROOT / "evidence" / "pt_fuzz_results.json"

DOC = "docs/MACHINE-READING-RESEARCH.md"
VALID = {
    "/reading/parse": {"path": DOC},
    "/reading/evidence": {"path": DOC, "limit": 5},
    "/reading/qa": {"question": "how many chunks", "k": 2},
    "/reading/summarize": {"query": "index chunks", "k": 2},
    "/reading/claim": {"claim": "The index holds 2403 chunks.", "k": 2},
    "/reading/scientific": {"path": DOC, "limit": 5},
    "/reading/contradictions": {"sentences": ["a b c d e", "a b c d f"]},
    "/kv/source": {"path": DOC},
    "/kv/peerreview": {"path": DOC},
    "/kv/reproducibility": {"path": DOC},
    "/kv/retraction": {"path": DOC},
    "/kv/consistency": {"paths": [DOC]},
    "/kv/factcheck": {"claim": "The index holds 2403 chunks.", "k": 2},
    "/kv/uncertainty": {"claim": "The index holds 2403 chunks.", "k": 2},
    "/ie/relations": {"text": "The kernel holds 219 modules."},
    "/ie/events": {"text": "Patients diagnosed with hypertension were treated with lisinopril."},
    "/ie/temporal": {"text": "Measured on 2026-11-25 after 3 weeks."},
    "/ie/negation": {"text": "No fever, but cough is present."},
    "/ie/uncertainty": {"text": "The result may indicate an infection."},
    "/ie/coref": {"text": "The kernel holds the registry. It enforces the guard."},
    "/ie/entities": {"text": "Staphylococcus aureus was screened."},
    "/ie/language": {"text": "The kernel holds the registry and enforces the guard."},
    "/ie/structured": {"text": "The kernel holds 219 modules."},
    "/reason/forward": {"rulebase": "druglikeness", "facts": [{"atom": "mw_gt_500", "source": "x"}]},
    "/reason/abduce": {"rulebase": "druglikeness", "observation": "violation_mw",
                       "hypotheses": [{"atom": "mw_gt_500"}], "max_size": 1},
    "/reason/analogy": {"cases": [{"id": "a", "features": [0.0], "outcome": 0.0},
                                  {"id": "b", "features": [1.0], "outcome": 1.0},
                                  {"id": "c", "features": [2.0], "outcome": 2.0},
                                  {"id": "d", "features": [3.0], "outcome": 3.0}],
                        "mode": "query", "query": [1.5]},
}


def raw_post(path, body: bytes, timeout=60):
    req = urllib.request.Request(BASE+path, data=body, method="POST",
                                 headers={"Content-Type": "application/json", "X-Council-Token": TOK})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, r.read()[:200]
    except urllib.error.HTTPError as e:
        return e.code, e.read()[:200]
    except Exception as e:
        return 0, str(e).encode()[:120]


def payloads(base: dict):
    """Hostile variants of a valid body. Each is (label, bytes)."""
    deep = b'{"a":' * 200 + b'1' + b'}' * 200
    huge = json.dumps({"text": "A" * 2_000_000, "path": DOC}).encode()
    traversal = json.dumps({**base, "path": "../../../../Windows/win.ini"}).encode()
    abs_path = json.dumps({**base, "path": "C:/Windows/win.ini"}).encode()
    nul = json.dumps({**base, "path": DOC + "\u0000.txt"}).encode()
    rtl = json.dumps({**base, "path": DOC + "\u202e"}).encode()
    big_k = json.dumps({**base, "k": 2**63 - 1}).encode()
    neg_k = json.dumps({**base, "k": -5}).encode()
    float_k = json.dumps({**base, "k": 1.5}).encode()
    str_k = json.dumps({**base, "k": "many"}).encode()
    nan = b'{"k": NaN, "path": "' + DOC.encode() + b'"}'
    inf = b'{"k": Infinity, "path": "' + DOC.encode() + b'"}'
    nulls = json.dumps({k: None for k in base}).encode()
    arrays = json.dumps({k: [1, 2, 3] for k in base}).encode()
    nested = json.dumps({k: {"x": {"y": {}}} for k in base}).encode()
    # type confusion: every field becomes the wrong type in turn
    confusions = []
    for k in base:
        b = dict(base)
        b[k] = 12345 if not isinstance(base[k], int) else "not-a-number"
        confusions.append((f"type-confusion:{k}", json.dumps(b).encode()))
    return [
        ("empty-body", b""),
        ("invalid-json", b"{not json"),
        ("truncated-json", b'{"path": '),
        ("json-null", b"null"),
        ("json-array", b"[1,2,3]"),
        ("json-string", b'"hello"'),
        ("deep-nesting", deep),
        ("huge-string", huge),
        ("path-traversal", traversal),
        ("absolute-path", abs_path),
        ("nul-in-path", nul),
        ("rtl-override", rtl),
        ("k-overflow", big_k),
        ("k-negative", neg_k),
        ("k-float", float_k),
        ("k-string", str_k),
        ("nan", nan),
        ("infinity", inf),
        ("all-null", nulls),
        ("all-arrays", arrays),
        ("all-nested-objects", nested),
        ("utf8-invalid", b'{"path": "\\xff\\xfe"}'),
        ("bom-prefix", b"\xef\xbb\xbf" + json.dumps(base).encode()),
        ("dup-keys", b'{"path": "a", "path": "' + DOC.encode() + b'"}'),
    ] + confusions


def main():
    results = []
    panics = []
    for path, base in VALID.items():
        code, _ = raw_post(path, json.dumps(base).encode())
        results.append({"endpoint": path, "payload": "baseline", "status": code})
        for label, body in payloads(base):
            code, snippet = raw_post(path, body)
            results.append({"endpoint": path, "payload": label, "status": code})
            if code >= 500 or code == 0:
                panics.append({"endpoint": path, "payload": label, "status": code,
                               "detail": snippet.decode("utf-8", "replace")[:160]})
            elif code == 200 and label in ("path-traversal", "absolute-path", "nul-in-path"):
                panics.append({"endpoint": path, "payload": label, "status": code,
                               "detail": "ACCEPTED a hostile path (200)"})

    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps({"requests": len(results), "results": results, "defects": panics}, indent=1),
                   encoding="utf-8")

    print(f"endpoints fuzzed {len(VALID)} | requests {len(results)}")
    print(f"panics (5xx) or hostile-path acceptances: {len(panics)}")
    seen = set()
    for p in panics:
        key = (p["endpoint"], p["status"])
        if key in seen:
            continue
        seen.add(key)
        print(f"  {p['status']} {p['endpoint']:26} {p['payload']:20} {p['detail'][:80]}")
    print(f"wrote {OUT.relative_to(ROOT)}")
    print("RESULT:", "PASS" if not panics else "FAIL")
    return 0 if not panics else 1


if __name__ == "__main__":
    sys.exit(main())
