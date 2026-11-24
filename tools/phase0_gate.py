"""Phase 0 gate, run exactly as pre-registered in docs/TRM_Domain_Grounding_Plan_2026-11-24.md.

Gate: each of the four probe questions must return a top-1 chunk CONTAINING the key string.
Scored on v1 (the shipped index) and v2 (facts + prose) with the same embedder, same cosine, same probes,
so the delta is the index and nothing else. No threshold is tuned after seeing the numbers.
"""
import json
import urllib.request
import numpy as np

ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
IDX = {"v1 (shipped)": f"{ROOT}/data/council_evidence_index.json",
       "v2 (facts+prose)": f"{ROOT}/data/council_evidence_index_v2.json"}

PROBES = [
    ("which TCP port does the TRM council server listen on?", ["8022"]),
    ("what is axiom A2 about in the council's framework?", ["free will", "consent"]),
    ("does disabling TLS certificate verification allow machine-in-the-middle attacks?", ["machine-in-the-middle", "mitm"]),
    ("why is an append-only hash chain tamper-evident?", ["hash chain", "previous hash", "prior hash"]),
]


def embed(text):
    body = json.dumps({"model": "nomic-embed-text", "input": [text]}).encode()
    r = urllib.request.Request("http://127.0.0.1:8593/v1/embeddings", data=body,
                               headers={"Content-Type": "application/json"})
    v = np.array(json.loads(urllib.request.urlopen(r, timeout=90).read())["data"][0]["embedding"],
                 dtype=np.float32)
    return v / (np.linalg.norm(v) + 1e-9)


print(f"  {'index':18} {'probe':58} top1   key in top1?  key in top3?")
summary = {}
for label, path in IDX.items():
    d = json.load(open(path))
    chunks = d["chunks"]
    V = np.array([c["vec"] for c in chunks], dtype=np.float32)
    V /= (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)
    passed = 0
    for q, keys in PROBES:
        s = V @ embed(q)
        order = np.argsort(-s)[:3]
        t1 = chunks[order[0]]["text"].lower()
        t3 = " ".join(chunks[i]["text"].lower() for i in order)
        in1 = any(k in t1 for k in keys)
        in3 = any(k in t3 for k in keys)
        passed += in1
        print(f"  {label:18} {q[:56]:58} {s[order[0]]:.3f}  {'PASS' if in1 else 'miss':11}  "
              f"{'yes' if in3 else 'no':3}   {chunks[order[0]]['source'][:34]}")
    summary[label] = passed
print(f"\n  GATE: v1 {summary['v1 (shipped)']}/4, v2 {summary['v2 (facts+prose)']}/4 "
      f"(pre-registered requirement: v2 must be 4/4)")
print("  VERDICT:", "PASS" if summary["v2 (facts+prose)"] == 4 else "FAIL")
