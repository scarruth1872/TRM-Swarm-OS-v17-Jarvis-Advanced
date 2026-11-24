"""Phase 1 gate — as written in docs/TRM_Domain_Grounding_Plan_2026-11-24.md, run unmodified.

    "on the 12 dev tasks, every phoenix_evidence entry must contain the text its verdict cites
     (provenance check, machine-verifiable)"

Method: run each dev task through POST /council/propose, wait for settlement, then for every citation of the
form "[source] snippet" require that a chunk with THAT source contains THAT snippet in the index the ranker
read (continuum_v3_chunks.jsonl, the same rows /continuum/search serves). A citation that names a source
whose text does not contain the quoted snippet fails, as does a citation with no matching source at all.

Also recorded, without gating: verdict, vote tallies, evidence_used, and which ranker answered.
"""
import json
import re
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:8022"
ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"
OUT = S + "phase1_provenance_results.jsonl"

chunks = [json.loads(l) for l in open(f"{ROOT}/data/continuum_v3_chunks.jsonl", encoding="utf-8")]
by_source = {}
for c in chunks:
    by_source.setdefault(c["source"], []).append(" ".join(c["text"].split()))


def norm(s):
    return " ".join(s.split())


def post(path, payload):
    r = urllib.request.Request(BASE + path, data=json.dumps(payload).encode(),
                               headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(r, timeout=90).read())


def get(path):
    return json.loads(urllib.request.urlopen(BASE + path, timeout=60).read())


tasks = json.load(open(S + "blind_prereg.json"))["tasks"]
rows = []
with open(OUT, "a", encoding="utf-8") as fh:
    for t in tasks:
        n, text, half = t["n"], t["text"], t["half"]
        t0 = time.time()
        try:
            pid = post("/council/propose", {"topic": text})["id"]
        except Exception as e:
            print(f"  dev{n:2} POST failed: {str(e)[:80]}")
            continue
        d = {}
        for _ in range(50):
            d = get(f"/council/proposal/{pid}")
            if d.get("phase") not in ("EVALUATING", "PENDING", None):
                break
            time.sleep(6)
        ev = d.get("phoenix_evidence") or []
        verified = failed = 0
        miss = []
        for entry in ev:
            m = re.match(r"^\[([^\]]+)\]\s*(.*)$", norm(str(entry)))
            if not m:
                failed += 1
                miss.append(("unparsed", str(entry)[:80]))
                continue
            src, snip = m.group(1).strip(), m.group(2).strip()
            pool = by_source.get(src)
            if not pool:
                failed += 1
                miss.append(("no such source", src))
                continue
            if any(snip[:120] in p for p in pool):
                verified += 1
            else:
                failed += 1
                miss.append(("snippet absent", f"{src} :: {snip[:70]}"))
        rec = {"set": "dev", "n": n, "text": text, "half": half, "proposal_id": pid,
               "phase": d.get("phase"), "a": d.get("approve_count"), "r": d.get("reject_count"),
               "ab": d.get("abstain_count"), "evidence_used": d.get("evidence_used"),
               "evidence_ranker": d.get("evidence_ranker"), "deliberation_mode": d.get("deliberation_mode"),
               "citations": len(ev), "verified": verified, "failed": failed, "misses": miss[:6],
               "elapsed_s": round(time.time() - t0, 1)}
        rows.append(rec)
        fh.write(json.dumps(rec) + "\n")
        fh.flush()
        print(f"  dev{n:2} {half:6} {str(rec['phase']):13} A{rec['a']}/R{rec['r']}/Ab{rec['ab']} "
              f"ev={rec['evidence_used']} ranker={rec['evidence_ranker']} "
              f"cit={len(ev)} ok={verified} bad={failed} {rec['elapsed_s']}s")

tot_c = sum(r["citations"] for r in rows)
tot_v = sum(r["verified"] for r in rows)
tot_f = sum(r["failed"] for r in rows)
print(f"\n  GATE: {tot_v}/{tot_c} citations verified, {tot_f} failed, over {len(rows)} proposals")
print(f"  VERDICT: {'PASS' if tot_c > 0 and tot_f == 0 else 'FAIL'}")
