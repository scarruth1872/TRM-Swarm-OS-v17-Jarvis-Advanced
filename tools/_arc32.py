import sys
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
import json
import pathlib
import torch

D = pathlib.Path(r"F:\Development sites\TRM agent swarm\kaggle\combined")
W = 32          # row-major canvas width
SEQ = 1024      # W*W
PAD = 10

ch = json.loads((D / "arc-agi_training_challenges.json").read_text(encoding="utf-8"))
so = json.loads((D / "arc-agi_training_solutions.json").read_text(encoding="utf-8"))

from swarm_v2.core.trm_brain import TRMBrain

b = TRMBrain()
b._ensure_loaded()

def enc(grid):
    """Place grid row-major on a 32-wide canvas, pad to 1024."""
    t = [PAD] * SEQ
    for r, row in enumerate(grid):
        for c, v in enumerate(row):
            i = r * W + c
            if i < SEQ:
                t[i] = v
    return t

def dec(tokens, h, w):
    return [[tokens[r * W + c] for c in range(w)] for r in range(h)]

task_ids = sorted(ch.keys())[:5]

for tid in task_ids:
    task = ch[tid]
    sol = so[tid]
    pr = task["train"][0]
    inp, out = pr["input"], pr["output"]
    oh, ow = len(out), len(out[0])

    toks = enc(inp)
    for pid in (0, 1, 2):
        r = b.reason(toks, puzzle_id=pid, full_output=True)
        preds = r[-1] if isinstance(r[0], list) else r
        g = dec(preds, oh, ow)
        match = sum(1 for r in range(oh) for c in range(ow) if g[r][c] == out[r][c])
        print(f"{tid} pid={pid} in={len(inp)}x{len(inp[0])} out={oh}x{ow} match={match}/{oh*ow}")
    print(f"  expected row0: {out[0][:10]}")
    print(f"  pred     row0: {dec((lambda r: r[-1] if isinstance(r[0], list) else r)(b.reason(toks, puzzle_id=0, full_output=True)), oh, ow)[0][:10]}")
    print()
