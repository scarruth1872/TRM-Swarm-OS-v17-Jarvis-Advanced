import sys
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')
import json, pathlib, numpy as np
exec(open(r'F:\Development sites\TRM-Swarm-OS-v2\tools\_t.py').read().split('def main')[0])
from swarm_v2.core.trm_brain import TRMBrain
ch = json.loads((D/'arc-agi_training_challenges.json').read_text(encoding='utf-8'))
so = json.loads((D/'arc-agi_training_solutions.json').read_text(encoding='utf-8'))
b = TRMBrain(); b._ensure_loaded()
tid = '00d62c1b'
to = so[tid][0]; r, c = np.array(to).shape
toks = enc(ch[tid]['test'][0]['input']) + [0]*(1024-SEQ)
best = (0, -1)
for pid in range(0, 16):
    pr = b.reason(toks, puzzle_id=pid, full_output=True, output_len=1024)[-1]
    m = int((dec(pr, r, c) == np.array(to)).sum())
    print(f'pid={pid:>3}  match={m}/{r*c}  {100*m/(r*c):.1f}%')
    if m > best[0]: best = (m, pid)
print('BEST:', best)
