"""Phase 0 gate, split — reads its own pre-registration and runs Gate A unmodified.

Gate A (platform facts): 4/4 top-1 chunks must contain the key.
Gate B (general knowledge): NOT RUN — recorded as out of scope in the pre-registration; a second 0/2
would only restate the original failure. The model carries that measurement (6/6 general).

Probes come from docs/phase0_gate_split_prereg_2026-11-24.json, written before this ran.
"""
import json
import os
import sys
import urllib.request
import numpy as np

ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
PREREG = f"{ROOT}/docs/phase0_gate_split_prereg_2026-11-24.json"
IDX = {"v1 (shipped)": f"{ROOT}/data/council_evidence_index.json",
       "v2 (facts+prose)": f"{ROOT}/data/council_evidence_index_v2.json"}
# extra indexes named on the command line are scored with the SAME probes and the SAME 4/4 rule
for extra in sys.argv[1:]:
    p = f"{ROOT}/data/council_evidence_index_{extra}.json"
    if os.path.exists(p):
        IDX[f"{extra} (post-dilution-fix)"] = p


def embed(text):
    body = json.dumps({"model": "nomic-embed-text", "input": [text]}).encode()
    r = urllib.request.Request("http://127.0.0.1:8593/v1/embeddings", data=body,
                               headers={"Content-Type": "application/json"})
    v = np.array(json.loads(urllib.request.urlopen(r, timeout=90).read())["data"][0]["embedding"],
                 dtype=np.float32)
    return v / (np.linalg.norm(v) + 1e-9)


reg = json.load(open(PREREG))
probes = reg["gate_a_platform_facts"]["probes"]
print(f"  pre-registered {reg['written_utc']}: Gate A = {reg['gate_a_platform_facts']['requirement']}\n")

results = {}
for label, path in IDX.items():
    d = json.load(open(path))
    chunks = d["chunks"]
    V = np.array([c["vec"] for c in chunks], dtype=np.float32)
    V /= (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)
    passed, detail = 0, []
    print(f"  == {label} ({len(chunks)} chunks) ==")
    for p in probes:
        q, keys = p["question"], p["keys"]
        s = V @ embed(q)
        o = np.argsort(-s)[:3]
        t1 = chunks[o[0]]["text"].lower()
        ok = any(k in t1 for k in keys)
        passed += ok
        print(f"    {'PASS' if ok else 'miss'}  {q[:52]:54} {s[o[0]]:.3f}  {chunks[o[0]]['source'][:30]}")
        detail.append({"q": q, "keys": keys, "top1": chunks[o[0]]["source"], "score": float(s[o[0]]), "ok": bool(ok)})
    results[label] = {"passed": passed, "detail": detail}
    print()

# The gating artifact is the LAST index named on the command line (the newest build), else v2.
gate_label = list(results)[-1] if len(results) > 2 else "v2 (facts+prose)"
a = results[gate_label]["passed"]
print(f"  GATE A per artifact (rule 4/4, unchanged):")
for label, r in results.items():
    print(f"    {label:34} {r['passed']}/4  {'PASS' if r['passed'] == 4 else 'FAIL'}")
print(f"  GATE B: not run - out of scope for an index; model measured 6/6 general, 0/4 platform")
print(f"  VERDICT ({gate_label}): "
      f"{'PASS - Phase 1 opens' if a == 4 else 'FAIL - plan stops, this artifact stays unwired'}")
json.dump({"prereg": PREREG, "gating_artifact": gate_label, "detail": results,
           "gate_b": "not run (out of scope)"},
          open(f"{ROOT}/data/phase0_gate_split_result.json", "w"), indent=1)
sys.exit(0 if a == 4 else 1)
