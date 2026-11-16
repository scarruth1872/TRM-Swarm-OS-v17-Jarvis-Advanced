import sys; sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

import json, pathlib
sys_root = r"F:\Development sites\TRM-Swarm-OS-v2"
D = pathlib.Path(r"F:\Development sites\TRM agent swarm\kaggle\combined")
ch = json.loads((D/"arc-agi_training_challenges.json").read_text(encoding="utf-8"))
so = json.loads((D/"arc-agi_training_solutions.json").read_text(encoding="utf-8"))
from swarm_v2.core.trm_brain import TRMBrain
b = TRMBrain(); b._ensure_loaded()

k = "007bbfb7"
keys = sorted(ch)
idx = keys.index(k)
inp = ch[k]["train"][0]["input"]
exp = so[k][0]
flat = [c for row in inp for c in row]
exp_flat = [c for row in exp for c in row]
print("task", k, "sorted idx:", idx, "| expected 81 flat:", exp_flat[:12])

# does puzzle_id change the output at all?
base = None
for pid in [0, idx, idx+1, 1000]:
    out = b.reason(flat, puzzle_id=pid, full_output=True)[0]
    match = sum(1 for a,bb in zip(out[:81], exp_flat) if a==bb)
    tag = "SAME" if base is None else ("same" if out[:81]==base else "DIFF")
    if base is None: base = out[:81]
    print(f"  puzzle_id={pid:<7} match={match}/81  {tag}  first9={out[:9]}")
