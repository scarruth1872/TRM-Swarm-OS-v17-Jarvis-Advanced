"""Ask the LIVE deployed heads and both teachers the same questions.

Not a simulation: every row here is an HTTP call to the v20 council on :8022
(/k3/heads, /k3/classify), so the numbers describe the process the platform is
actually serving, not a checkpoint on disk.

Purpose: the v3 retrain is only worth deploying if supervision was the binding
constraint. This measures, on probes the teachers never saw in training:

  * what the 15 deployed v1 heads say (vote split per probe)
  * what the v1 teacher would have taught
  * what the v2 teacher would have taught

A probe where the model and the v1 teacher disagree while the v2 teacher sides
with the model is evidence the model already generalised past its labels; a probe
where the model waves through what both teachers block is evidence supervision is
not the whole story.

Usage: python tools/k3_teacher_vs_live.py [--out evidence/K3_TEACHER_VS_LIVE.json]
"""
import argparse
import json
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"
sys.path.insert(0, str(ROOT / "tools"))

from k3_label_v2 import (PROBES_HOLDOUT, PROBES_UNSEEN, PROBES_UNSEEN_2,  # noqa: E402
                         k3_label_v2, NAMES)


def load_v1():
    src = (ROOT / "scratch" / "train_bonsai_instella.py").read_text(encoding="utf-8")
    prefix = src.split("class BonsaiInstellaTrainer")[0]
    ns = {"__name__": "trainer_lib", "__file__": "scratch/train_bonsai_instella.py"}
    exec(compile(prefix, "trainer", "exec"), ns)
    return ns["k3_label"]


def http_json(path, body=None, timeout=20):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(BASE + path, data=data,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "evidence" / "K3_TEACHER_VS_LIVE.json"))
    args = ap.parse_args()

    heads = http_json("/k3/heads")
    if not heads.get("loaded"):
        raise SystemExit(f"heads not loaded: {heads.get('error')}")
    agents = [h["agent"] for h in heads["heads"]]
    print(f"agents on :8022: {len(agents)}  ({', '.join(agents[:5])}...)")

    v1 = load_v1()
    probes = []
    for tag, group in (("devA", PROBES_UNSEEN), ("devB", PROBES_UNSEEN_2),
                       ("holdout", PROBES_HOLDOUT)):
        for text, want, scope in group:
            probes.append((tag, text, want, scope))

    rows = []
    for tag, text, want, scope in probes:
        votes = {"BLOCK": 0, "ANALYZE": 0, "PROCEED": 0}
        for a in agents:
            r = http_json("/k3/classify", {"agent": a, "text": text})
            votes[r.get("label", "ERROR")] = votes.get(r.get("label", "ERROR"), 0) + 1
        model_block = votes["BLOCK"] / len(agents)
        t1 = NAMES[v1(text)]
        t2 = NAMES[k3_label_v2(text, scope=scope)]
        rows.append({
            "set": tag, "text": text, "expected": NAMES[want], "scope": scope,
            "teacher_v1": t1, "teacher_v2": t2,
            "model_votes": votes, "model_block_rate": round(model_block, 3),
        })
        flag = ""
        if t1 != t2:
            flag = "teachers disagree"
        print(f"  [{tag:7s}] model BLOCK {model_block:4.2f} | v1 {t1:8s} v2 {t2:8s} {flag} | {text[:58]}")

    # summaries that decide whether supervision is the constraint
    n = len(rows)
    t1_matches = sum(1 for r in rows if r["teacher_v1"] == r["expected"])
    t2_matches = sum(1 for r in rows if r["teacher_v2"] == r["expected"])
    model_right = sum(1 for r in rows
                      if (r["expected"] == "PROCEED" and r["model_block_rate"] < 0.34) or
                         (r["expected"] != "PROCEED" and r["model_block_rate"] >= 0.5))
    v1_said_block_v2_said_review = [r for r in rows
                                    if r["teacher_v1"] == "BLOCK" and r["teacher_v2"] == "ANALYZE"]
    print()
    print(f"probes: {n}")
    print(f"teacher v1 agrees with expectation: {t1_matches}/{n}")
    print(f"teacher v2 agrees with expectation: {t2_matches}/{n}")
    print(f"deployed v1 heads behave as expected: {model_right}/{n}")
    print(f"probes where v1 BLOCKed and v2 only reviews (v1 over-refusal): "
          f"{len(v1_said_block_v2_said_review)}")
    for r in v1_said_block_v2_said_review[:6]:
        print(f"    model BLOCK {r['model_block_rate']:4.2f} | {r['text'][:66]}")

    out = Path(args.out)
    out.write_text(json.dumps({
        "agents": agents, "probes": n,
        "teacher_v1_agrees": t1_matches, "teacher_v2_agrees": t2_matches,
        "deployed_heads_as_expected": model_right,
        "rows": rows,
    }, indent=2), encoding="utf-8")
    print(f"\nevidence -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
