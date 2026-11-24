"""v4 relevance round — reads docs/phase1_relevance_prereg_v4_2026-11-24.json and applies it unmodified.

Order of operations is fixed by the pre-registration:
  1. CONTAMINATION CHECK FIRST. If any of the 12 task texts appears in the built index, ABORT and score
     nothing. A round that can cite its own questions is void by construction, not by exclusion list.
  2. Run the 12 tasks through /council/propose on the v4 continuum.
  3. Secondary (not gating): provenance of each citation, and distinct-sources-per-proposal.
  4. Relevance judged by :8587, topic + snippet only, verdict withheld. PASS = 18/24 DIRECT.
"""
import json
import re
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:8022"
JUDGE = "http://127.0.0.1:8587/v1/chat/completions"
ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"
PREREG = f"{ROOT}/docs/phase1_relevance_prereg_v4_2026-11-24.json"

pre = json.load(open(PREREG))
tasks = pre["tasks"]
print(f"  prereg {pre['written_at_utc']}  rule: {pre['decision_rule']['PASS']}")

# ---------- 1. contamination check ----------
chunks = [json.loads(l) for l in open(f"{ROOT}/data/continuum_v4_chunks.jsonl", encoding="utf-8")]
flat = " ".join(" ".join(c["text"].split()).lower() for c in chunks)
hits = []
for t in tasks:
    probe = " ".join(t["text"].split()).lower()[:60]
    if probe in flat:
        hits.append(t["n"])
if hits:
    print(f"  ABORT: tasks {hits} appear inside the v4 index — the round is void")
    sys.exit(2)
print(f"  contamination check: 0 of {len(tasks)} task texts found in {len(chunks)} chunks — clean")

by_source = {}
for c in chunks:
    by_source.setdefault(c["source"], []).append(" ".join(c["text"].split()))

RUBRIC = (
    "You are grading retrieval quality, not style. Read the PROPOSAL and the CITED TEXT, then choose exactly "
    "one label:\n"
    "DIRECT - the cited text names or discusses the proposal's actual subject matter\n"
    "TANGENTIAL - same domain or vocabulary, but does not address the specific subject\n"
    "UNRELATED - about something else entirely; citing it would mislead a reader\n"
    "Answer in exactly this form on one line: LABEL: <DIRECT|TANGENTIAL|UNRELATED> - <six words why>"
)


def judge(topic, snippet):
    body = json.dumps({"model": "gpt-oss-20b", "temperature": 0.0, "max_tokens": 600,
                       "reasoning_effort": "low",
                       "messages": [{"role": "system", "content": RUBRIC},
                                    {"role": "user", "content": f"PROPOSAL: {topic}\n\nCITED TEXT: {snippet}"}]}).encode()
    r = urllib.request.Request(JUDGE, data=body, headers={"Content-Type": "application/json"})
    msg = json.loads(urllib.request.urlopen(r, timeout=200).read())["choices"][0]["message"]
    for src in (msg.get("content") or "", msg.get("reasoning_content") or ""):
        m = re.search(r"LABEL:\s*(DIRECT|TANGENTIAL|UNRELATED)", src.upper())
        if m:
            return m.group(1), src.strip()[:160]
    m = re.search(r"\b(DIRECT|TANGENTIAL|UNRELATED)\b", (msg.get("content") or "").upper())
    return (m.group(1) if m else "UNGRADED"), (msg.get("content") or "")[:160]


# ---------- 2. proposals ----------
rows = []
out = S + "phase1_v4_results.jsonl"
open(out, "w").close()
for t in tasks:
    t0 = time.time()
    r = urllib.request.Request(BASE + "/council/propose", data=json.dumps({"topic": t["text"]}).encode(),
                               headers={"Content-Type": "application/json"})
    pid = json.loads(urllib.request.urlopen(r, timeout=90).read())["id"]
    d = {}
    for _ in range(50):
        d = json.loads(urllib.request.urlopen(f"{BASE}/council/proposal/{pid}", timeout=60).read())
        if d.get("phase") not in ("EVALUATING", "PENDING", None):
            break
        time.sleep(6)
    ev = d.get("phoenix_evidence") or []
    cites, verified, srcs = [], 0, []
    for entry in ev:
        m = re.match(r"^\[([^\]]+)\]\s*(.*)$", " ".join(str(entry).split()))
        if not m:
            continue
        src, snip = m.group(1).strip(), m.group(2).strip()
        srcs.append(src)
        ok = any(snip[:120] in p for p in by_source.get(src, []))
        verified += ok
        cites.append({"source": src, "snippet": snip, "provenance_ok": bool(ok)})
    rec = {"n": t["n"], "half": t["half"], "text": t["text"], "id": pid, "phase": d.get("phase"),
           "a": d.get("approve_count"), "r": d.get("reject_count"), "ab": d.get("abstain_count"),
           "evidence_used": d.get("evidence_used"), "ranker": d.get("evidence_ranker"),
           "citations": cites, "provenance_ok": verified, "distinct_sources": len(set(srcs)),
           "elapsed_s": round(time.time() - t0, 1)}
    rows.append(rec)
    with open(out, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(rec) + "\n")
    print(f"  v4-{t['n']:2} {t['half']:6} {str(rec['phase']):13} A{rec['a']}/R{rec['r']}/Ab{rec['ab']} "
          f"cit={len(cites)} prov={verified}/{len(cites)} distinct={rec['distinct_sources']} {rec['elapsed_s']}s")

# ---------- 3+4. relevance ----------
print("\n  judging relevance (topic + snippet only, verdict withheld)")
items = []
for rec in rows:
    for c in rec["citations"]:
        label, raw = judge(rec["text"], c["snippet"])
        items.append({**c, "n": rec["n"], "half": rec["half"], "topic": rec["text"],
                      "label": label, "judge_raw": raw})
        print(f"    v4-{rec['n']:2} {label:10} {c['source'][:44]}")

n = len(items)
direct = sum(1 for i in items if i["label"] == "DIRECT")
tang = sum(1 for i in items if i["label"] == "TANGENTIAL")
unrel = sum(1 for i in items if i["label"] == "UNRELATED")
ungr = sum(1 for i in items if i["label"] == "UNGRADED")
prov_ok = sum(r["provenance_ok"] for r in rows)
prov_all = sum(len(r["citations"]) for r in rows)
src_max = max((r["distinct_sources"] for r in rows), default=0)

print(f"\n  DIRECT {direct}/{n}   TANGENTIAL {tang}   UNRELATED {unrel}   UNGRADED {ungr}")
print(f"  provenance {prov_ok}/{prov_all}   max distinct sources in one proposal: {src_max}")
if ungr >= 3:
    verdict = "INVALID"
elif direct >= 18:
    verdict = "PASS"
else:
    verdict = "FAIL"
print(f"  VERDICT: {verdict}")

json.dump({"prereg": pre["written_at_utc"], "index": "v4 (2403 chunks, eval docs excluded)",
           "contamination": 0, "n": n, "direct": direct, "tangential": tang, "unrelated": unrel,
           "ungraded": ungr, "provenance_ok": prov_ok, "provenance_total": prov_all,
           "max_distinct_sources": src_max, "verdict": verdict,
           "proposals": rows, "items": items},
          open(f"{ROOT}/docs/phase1_relevance_v4_results_2026-11-24.json", "w"), indent=1)
print("  wrote docs/phase1_relevance_v4_results_2026-11-24.json")
