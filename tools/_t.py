import sys
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
import json, pathlib
import numpy as np

D = pathlib.Path(r"F:\Development sites\TRM agent swarm\kaggle\combined")
G = 30
SEQ = G * G
def enc(g):
    a = np.array(g, dtype=np.uint8)
    r, c = a.shape
    b = np.zeros((G, G), dtype=np.uint8)
    b[:r, :c] = a + 2
    b[r, :c] = 1
    b[:r, c] = 1
    return b.flatten().tolist()
def dec(t, r, c):
    a = np.array(t[:SEQ], dtype=np.int64).reshape(G, G)
    o = a[:r, :c] - 2
    return np.where((o >= 0) & (o <= 9), o, 0)

def main():
    ch = json.loads((D/'arc-agi_training_challenges.json').read_text(encoding='utf-8'))
    so = json.loads((D/'arc-agi_training_solutions.json').read_text(encoding='utf-8'))
    from swarm_v2.core.trm_brain import TRMBrain
    b = TRMBrain(); b._ensure_loaded()
    for tid in sorted(ch)[:5]:
        t = ch[tid]; sol = so[tid]
        ti = t['test'][0]['input']; to = sol[0]
        r, c = np.array(to).shape
        toks = enc(ti) + [0]*(1024-SEQ)
        for pid in (0, 1):
            pr = b.reason(toks, puzzle_id=pid, full_output=True, output_len=1024)[-1]
            d = dec(pr, r, c)
            m = int((d == np.array(to)).sum())
            print(f'{tid} {r}x{c} pid={pid} match={m}/{r*c}')

if __name__ == '__main__': main()
