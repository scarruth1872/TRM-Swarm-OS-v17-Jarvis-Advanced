import sys; sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

import json, pathlib
D = pathlib.Path(r"F:\Development sites\TRM agent swarm\kaggle\combined")
ch = json.loads((D/"arc-agi_training_challenges.json").read_text(encoding="utf-8"))
so = json.loads((D/"arc-agi_training_solutions.json").read_text(encoding="utf-8"))
from swarm_v2.core.trm_brain import TRMBrain
b = TRMBrain(); b._ensure_loaded()

k = "007bbfb7"
inp = ch[k]["train"][0]["input"]
exp = so[k][0]
print("input 3x3:", inp)
print("expected 9x9 first row:", exp[0])
flat = [c for row in inp for c in row]
print("flat tokens:", flat, "len", len(flat))
outs = b.reason(flat, puzzle_id=0, full_output=True)
pred = outs[0]
print("pred len:", len(pred))
print("pred[0:9]  :", pred[0:9])
print("pred[0:81] as 9x9 first 3 rows:", [pred[i*9:(i+1)*9] for i in range(3)])
print("expected 9x9 first 3 rows   :", exp[0:3])
