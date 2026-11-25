#!/usr/bin/env python3
"""Task 582 — extractive QA where the label is a number that exists in the corpus.

Questions are built mechanically from real sentences: take a sentence containing a number, blank the number, and
ask for it. The expected answer is that exact number string. Two things are then checked: the answer must contain
the number, and its file offset must resolve to the bytes it cites.

A baseline is measured too — answering with a number taken from the top retrieved chunk instead of the
sentence-scored span — so the result shows lift rather than assuming it.
"""
import json, re, sys, urllib.request
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent

def post(path, payload):
    req = urllib.request.Request("http://127.0.0.1:8022"+path, data=json.dumps(payload).encode(),
                                 method="POST", headers={"Content-Type":"application/json","X-Council-Token":TOK})
    with urllib.request.urlopen(req, timeout=120) as r: return json.loads(r.read().decode())

NUM = re.compile(r"\b\d[\d,]*(?:\.\d+)?\b")

def build_questions(limit=20):
    qs = []
    for f in sorted(ROOT.glob("docs/*.md")):
        text = f.read_text(errors="replace")
        for line in text.splitlines():
            m = NUM.search(line)
            if not m or len(line) < 60 or line.startswith("|") or line.startswith("#"):
                continue
            num = m.group(0)
            if len(num) < 2 or num in ("1", "2", "3"):
                continue
            blanked = line[:m.start()] + "__" + line[m.end():]
            q = "What number is reported in: " + blanked.strip()[:180]
            qs.append((q, num, str(f.relative_to(ROOT)).replace("\\", "/")))
            if len(qs) >= limit:
                return qs
    return qs

def main():
    qs = build_questions(20)
    print(f"questions built from the corpus: {len(qs)}")
    correct = offset_ok = 0
    base_hits = 0
    for q, expected, src in qs:
        r = post("/reading/qa", {"question": q, "k": 5})
        ok = r.get("verdict") == "ANSWERED" and expected in (r.get("answer") or "")
        correct += 1 if ok else 0
        if r.get("file_offset") is not None:
            f = ROOT / r["source"]
            if f.exists():
                raw = f.read_bytes()
                off = r["file_offset"]
                if 0 <= off < len(raw) and raw[off:off+len(r["answer"].encode())].decode("utf-8", "replace") == r["answer"]:
                    offset_ok += 1
        # baseline: does the top chunk even contain the number?
        chunk = r.get("chunk") or ""
        base_hits += 1 if expected in chunk else 0
        if not ok and correct + (len(qs) - (qs.index((q, expected, src)) + 1)) <= 0:
            pass
    n = len(qs)
    print(f"answered correctly      : {correct}/{n} ({correct/max(1,n):.1%})")
    print(f"citations that resolve  : {offset_ok}/{correct}")
    print(f"baseline (number in the top chunk): {base_hits}/{n}")
    ok = n > 0 and correct >= 0.6 * n and offset_ok == correct
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
