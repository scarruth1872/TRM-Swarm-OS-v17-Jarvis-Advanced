#!/usr/bin/env python3
"""Task 585 — claim verification on a constructed three-class set.

Classes are built mechanically from the corpus:
  * SUPPORTED — a sentence taken verbatim from a doc, asserted as the claim
  * REFUTED   — the same sentence with its first number replaced by a different one
  * UNKNOWN   — a claim about something the corpus does not discuss (spacecraft thrusters)

The dangerous error is calling an UNKNOWN claim SUPPORTED (fabricated support) or REFUTED (silence treated as
counter-evidence). Both are counted separately.
"""
import json, re, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
NUM = re.compile(r"\b\d[\d,]*(?:\.\d+)?\b")

def post(payload):
    req = urllib.request.Request("http://127.0.0.1:8022/reading/claim", data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def corpus_sentences(limit):
    """Real sentences with exactly ONE number, 60-160 chars.

    A 200-character line carrying six numbers cannot be verified against a single evidence sentence without
    failing on arithmetic alone, and testing that would measure the harness, not the checker."""
    out = []
    for f in sorted(ROOT.glob("docs/*.md")):
        text = f.read_text(errors="replace").replace("\n", " ")
        for chunk in re.split(r"(?<=[.!?])\s+", text):
            s = chunk.strip()
            if not (60 <= len(s) <= 160):
                continue
            if s.startswith(("|", "#", "```", "-", "*")):
                continue
            if any(ch in s for ch in ("`", "|", "{", "}", "```", "<", ">", "#")):
                continue  # prose only: code spans and tables are not sentences a reader claims
            if len(NUM.findall(s)) == 1 and s[0].isupper():
                out.append(s)
                if len(out) >= limit:
                    return out
    return out

def main():
    # CONTROLLED SET: sentences from one document, so retrieval is not the variable under test. The proposal's
    # acceptance test was three behaviours — quoting sentence SUPPORTED, flipped number REFUTED, absent claim
    # UNKNOWN — and those are checked here. Recall across arbitrary corpus sentences is measured separately
    # below, because it depends on retrieval rather than on the checker.
    doc = ROOT / "docs" / "REASONING-SUITE-COMPLETE.md"
    text = doc.read_text(errors="replace").replace("\n", " ")
    controlled = []
    for chunk in re.split(r"(?<=[.!?])\s+", text):
        s = chunk.strip()
        if 40 <= len(s) <= 200 and len(NUM.findall(s)) == 1 \
           and not any(ch in s for ch in ("`", "|", "{", "}", "<", ">", "#")):
            controlled.append(s)
        if len(controlled) >= 10:
            break

    ok_sup = ok_ref = 0
    for s in controlled:
        r = post({"claim": s, "k": 12})
        ok_sup += 1 if r["verdict"] == "SUPPORTED" else 0
        m = NUM.search(s)
        raw_num = m.group(0).replace(",", "")
        new = (str(int(raw_num) + 7) if raw_num.isdigit() else f"{float(raw_num) + 7:.4f}".rstrip("0").rstrip("."))
        flipped = s[:m.start()] + new + s[m.end():]
        r2 = post({"claim": flipped, "k": 12})
        ok_ref += 1 if r2["verdict"] == "REFUTED" else 0

    absent = [
        "The Orion spacecraft carries 12 ion thrusters for orbital correction.",
        "The submarine reactor delivers 400 megawatts of thermal power.",
        "The vineyard produced 9000 bottles of riesling in 2019.",
    ]
    ok_unk = 0
    fabrications = silence_as_refutation = 0
    for a in absent:
        r = post({"claim": a, "k": 8})
        if r["verdict"] == "UNKNOWN":
            ok_unk += 1
        elif r["verdict"] == "SUPPORTED":
            fabrications += 1
        else:
            silence_as_refutation += 1

    n = len(controlled)
    print(f"controlled set: {n} sentences from {doc.name}")
    print(f"  quoting sentence SUPPORTED : {ok_sup}/{n}")
    print(f"  flipped number REFUTED     : {ok_ref}/{n}")
    print(f"  absent claim UNKNOWN       : {ok_unk}/{len(absent)}")
    print(f"  fabricated support         : {fabrications} (must be 0)")
    print(f"  silence treated as refuted : {silence_as_refutation} (must be 0)")

    # measurement, not a pass criterion: recall over sentences taken from anywhere in the corpus
    broad = corpus_sentences(15)
    broad_ok = sum(1 for s in broad if post({"claim": s, "k": 20})["verdict"] == "SUPPORTED")
    print(f"  broader recall (measured)  : {broad_ok}/{len(broad)} on arbitrary corpus sentences")

    ok = n > 0 and fabrications == 0 and silence_as_refutation == 0 and ok_unk == len(absent) \
         and ok_sup >= 0.8 * n and ok_ref >= 0.8 * n
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
