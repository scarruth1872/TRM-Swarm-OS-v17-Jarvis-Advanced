#!/usr/bin/env python3
"""Task 589 — contradiction detection: precision and recall on constructed pairs, reported separately.

Positives are built by flipping one number in a real sentence (a genuine contradiction on the same subject).
Negatives are pairs of unrelated real sentences (no shared subject). Precision and recall are both reported,
because a detector that finds every contradiction by calling everything one is useless and a detector that
finds none is worse.
"""
import json, re, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
NUM = re.compile(r"\b\d[\d,]*(?:\.\d+)?\b")

def post(payload):
    req = urllib.request.Request("http://127.0.0.1:8022/reading/contradictions", data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

def main():
    sents = []
    for f in sorted(ROOT.glob("docs/*.md")):
        for line in f.read_text(errors="replace").splitlines():
            if len(line) > 90 and not line.startswith(("|", "#", "```", "-")) and NUM.search(line):
                sents.append(line.strip())
                if len(sents) >= 12:
                    break
        if len(sents) >= 12:
            break

    pos_pairs = []
    for s in sents:
        m = NUM.search(s)
        if not m:
            continue
        flipped = s[:m.start()] + str(int(m.group(0).replace(",", "")) + 11) + s[m.end():]
        pos_pairs.append((s, flipped))

    neg_pairs = []
    for i in range(0, len(sents) - 1, 2):
        neg_pairs.append((sents[i], sents[i + 1]))

    # one call with all sentences: the handler reports every contradicting pair
    all_sents = []
    for a, b in pos_pairs: all_sents += [a, b]
    for a, b in neg_pairs: all_sents += [a, b]
    res = post({"sentences": all_sents})
    found = {(d["a"]["index"], d["b"]["index"]) for d in res["detail"]}

    # index layout: positives occupy consecutive pairs starting at 0, then negatives
    hits = 0
    for k in range(len(pos_pairs)):
        i, j = 2 * k, 2 * k + 1
        if (i, j) in found or (j, i) in found:
            hits += 1
    offset = 2 * len(pos_pairs)
    false_pos = 0
    for k in range(len(neg_pairs)):
        i, j = offset + 2 * k, offset + 2 * k + 1
        if (i, j) in found or (j, i) in found:
            false_pos += 1

    recall = hits / max(1, len(pos_pairs))
    precision = hits / max(1, hits + false_pos)
    print(f"positives {len(pos_pairs)} | detected {hits} | recall {recall:.2%}")
    print(f"negatives {len(neg_pairs)} | falsely flagged {false_pos}")
    print(f"precision {precision:.2%}")
    ok = len(pos_pairs) > 0 and recall >= 0.7 and false_pos == 0
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
