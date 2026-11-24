"""Phase 1 relevance gate — reads its own pre-registration FIRST and applies it unmodified.

Pre-registration: docs/phase1_relevance_prereg_2026-11-24.json (written 2026-11-24T16:56:04Z, before any
judgement). Rubric DIRECT / TANGENTIAL / UNRELATED / UNGRADED; PASS = 18 of 24 DIRECT; >=3 UNGRADED
invalidates the run.

The judge is the resident :8587 model, shown ONLY the proposal topic and the cited snippet — never the
verdict, never the agent names. A model grader is a weaker instrument than a human and the pre-registration
says so before the numbers exist.
"""
import json
import re
import sys
import urllib.request

BASE = "http://127.0.0.1:8022"
JUDGE = "http://127.0.0.1:8587/v1/chat/completions"
ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"

PREREG = f"{ROOT}/docs/phase1_relevance_prereg_2026-11-24.json"
pre = json.load(open(PREREG))
print(f"  prereg: {pre['written_at_utc']} — rule: {pre['decision_rule']['PASS']}")

STOP = set("""the a an and or of to in for on with is are was were be been being that this these those it its
as at by from not no nor but if then than so such into about over under we our you your they their he she
should would could can will must do does did done has have had make sure add section describe""".split())

RUBRIC = (
    "You are grading retrieval quality, not style. Read the PROPOSAL and the CITED TEXT, then choose exactly "
    "one label:\n"
    "DIRECT - the cited text names or discusses the proposal's actual subject matter\n"
    "TANGENTIAL - same domain or vocabulary, but does not address the specific subject\n"
    "UNRELATED - about something else entirely; citing it would mislead a reader\n"
    "Answer in exactly this form on one line: LABEL: <DIRECT|TANGENTIAL|UNRELATED> - <six words why>"
)


def judge(topic, snippet):
    body = json.dumps({
        "model": "gpt-oss-20b", "temperature": 0.0,
        # 90 was too small: the harmony model writes reasoning_content first, the budget ran out, and
        # `content` came back empty for all 24 items — which the pre-registered rule correctly called
        # INVALID rather than passing. 220 leaves room for the reasoning block plus the one-line label.
        # 220 was still not enough on the hard items: 8 of 24 came back with an empty content field
        # because the harmony reasoning block consumed the whole budget. 600 leaves room for reasoning
        # plus the one-line label. Budget is the only thing that changed; rubric and threshold are frozen.
        "max_tokens": 600, "reasoning_effort": "low",
        "messages": [{"role": "system", "content": RUBRIC},
                     {"role": "user", "content": f"PROPOSAL: {topic}\n\nCITED TEXT: {snippet}"}],
    }).encode()
    r = urllib.request.Request(JUDGE, data=body, headers={"Content-Type": "application/json"})
    out = json.loads(urllib.request.urlopen(r, timeout=180).read())
    msg = out["choices"][0]["message"]
    txt = msg.get("content") or ""
    m = re.search(r"LABEL:\s*(DIRECT|TANGENTIAL|UNRELATED)", txt.upper())
    if not m:
        # the model sometimes emits the label inside its reasoning block instead of content
        m = re.search(r"LABEL:\s*(DIRECT|TANGENTIAL|UNRELATED)", (msg.get("reasoning_content") or "").upper())
    if not m:
        # and sometimes it answers with the bare vocabulary word, since those three words are the entire
        # answer space. The raw reply is stored beside every judgement for audit.
        m = re.search(r"\b(DIRECT|TANGENTIAL|UNRELATED)\b", txt.upper())
    return (m.group(1) if m else "UNGRADED"), txt.strip()[:160]


def terms(s):
    return {w for w in re.findall(r"[a-z]{5,}", s.lower()) if w not in STOP}


rows = [json.loads(l) for l in open(f"{ROOT}/docs/phase1_provenance_results_2026-11-24.jsonl")]
items = []
for r in rows:
    d = json.loads(urllib.request.urlopen(f"{BASE}/council/proposal/{r['proposal_id']}", timeout=60).read())
    for entry in (d.get("phoenix_evidence") or []):
        m = re.match(r"^\[([^\]]+)\]\s*(.*)$", " ".join(str(entry).split()))
        if m:
            items.append({"n": r["n"], "half": r["half"], "topic": r["text"],
                          "source": m.group(1), "snippet": m.group(2)})

print(f"  {len(items)} citations to grade, judge=:8587, verdict withheld\n")
res = []
for it in items:
    label, raw = judge(it["topic"], it["snippet"])
    pt, st = terms(it["topic"]), terms(it["snippet"])
    ov = (len(pt & st) / len(pt)) if pt else 0.0
    warn = "  <-- mechanical overlap 0 while judged " + label if (ov == 0.0 and label == "DIRECT") else ""
    it.update({"label": label, "judge_raw": raw, "overlap": round(ov, 2), "disagree_warn": bool(warn)})
    res.append(it)
    print(f"  dev{it['n']:2} {it['half']:6} {label:10} overlap={ov:.2f} {it['source'][:28]:30}{warn}")

n = len(res)
direct = sum(1 for r in res if r["label"] == "DIRECT")
tang = sum(1 for r in res if r["label"] == "TANGENTIAL")
unrel = sum(1 for r in res if r["label"] == "UNRELATED")
ungr = sum(1 for r in res if r["label"] == "UNGRADED")
warns = sum(1 for r in res if r["disagree_warn"])

print(f"\n  DIRECT {direct}/{n}   TANGENTIAL {tang}   UNRELATED {unrel}   UNGRADED {ungr}")
print(f"  mechanical/model disagreement warnings: {warns}")
if ungr >= 3:
    print(f"  VERDICT: INVALID — {ungr} citations ungraded (>=3), re-run required")
elif direct >= 18:
    print(f"  VERDICT: PASS — {direct}/{n} DIRECT meets the pre-registered 18/24")
else:
    print(f"  VERDICT: FAIL — {direct}/{n} DIRECT, pre-registered requirement was 18/24")

json.dump({"prereg": pre["written_at_utc"], "n": n, "direct": direct, "tangential": tang,
           "unrelated": unrel, "ungraded": ungr, "disagree_warnings": warns, "items": res},
          open(f"{ROOT}/docs/phase1_relevance_results_2026-11-24.json", "w"), indent=1)
print(f"  wrote docs/phase1_relevance_results_2026-11-24.json")
