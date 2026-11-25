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

"""591 + 595 — fact checking and its calibration.

Three constructed classes, and the numbers that matter: accuracy per class, and the CALIBRATION of the confidence
(Brier score + reliability bins). A confidence that is high on wrong answers is worse than no confidence, so the
Brier score is the headline number, not an afterthought.
"""
import re
NUM = re.compile(r"\b\d[\d,]*(?:\.\d+)?\b")

def sentences(n):
    out = []
    for f in sorted(ROOT.glob("docs/*.md")):
        text = f.read_text(errors="replace").replace("\n", " ")
        for chunk in re.split(r"(?<=[.!?])\s+", text):
            s = chunk.strip()
            if 60 <= len(s) <= 160 and len(NUM.findall(s)) == 1 and s[:1].isupper() \
               and not any(c in s for c in ("`", "|", "{", "}", "<", ">", "#")):
                out.append(s)
                if len(out) >= n: return out
    return out

def main():
    items = []  # (claim, label)
    for s in sentences(12):
        items.append((s, 1))
        m = NUM.search(s)
        raw = m.group(0).replace(",", "")
        new = (str(int(raw) + 9) if raw.isdigit() else f"{float(raw) + 9:.4f}".rstrip("0").rstrip("."))
        items.append((s[:m.start()] + new + s[m.end():], 0))
    for a in ("The Orion spacecraft carries 12 ion thrusters for orbital correction.",
              "The submarine reactor delivers 400 megawatts of thermal power.",
              "The vineyard produced 9000 bottles of riesling in 2019."):
        items.append((a, 0))

    rows = []  # (p or None, label, verdict)
    correct = 0
    for claim, label in items:
        r = post("/kv/factcheck", {"claim": claim, "k": 10})
        if "_error" in r:
            print("ENDPOINT ERROR", r); return 1
        pred = 1 if r["verdict"] == "SUPPORTED" else 0
        correct += 1 if pred == label else 0
        rows.append((r["confidence"], label, r["verdict"]))

    abstained = [r for r in rows if r[0] is None]
    scored = [r for r in rows if r[0] is not None]
    hidden_correct = sum(1 for p, y, _ in abstained if y == 1)

    print(f"claims {len(items)} | verdict accuracy {correct}/{len(items)} ({correct/len(items):.1%})")
    print(f"abstained (no evidence) {len(abstained)}/{len(rows)} | of those, actually TRUE: {hidden_correct}")
    if scored:
        brier = sum((p - y) ** 2 for p, y, _ in scored) / len(scored)
        print(f"Brier over the {len(scored)} non-abstained: {brier:.4f} (0 perfect, 0.25 = always 0.5)")
        bins = [(0.0, 0.2), (0.2, 0.4), (0.4, 0.6), (0.6, 0.8), (0.8, 1.01)]
        print(f"{'bin':12} {'n':>3} {'mean p':>7} {'observed':>9}")
        for lo, hi in bins:
            sel = [(p, y) for p, y, _ in scored if lo <= p < hi]
            if not sel: continue
            mp = sum(p for p, _ in sel) / len(sel)
            ob = sum(y for _, y in sel) / len(sel)
            print(f"{f'{lo:.1f}-{hi:.1f}':12} {len(sel):3d} {mp:7.3f} {ob:9.3f}")
    else:
        brier = 1.0
        print("Brier: no claim received a probability — the model abstained on everything, which is not a calibrated model but is at least honest")

    # PASS requires: decent accuracy, calibration no worse than a constant, and abstention that is not hiding
    # most of the true answers.
    ok = (correct >= 0.6 * len(items)) and (brier <= 0.25) and (len(abstained) <= 0.6 * len(rows))
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1
if __name__ == "__main__":
    sys.exit(main())
