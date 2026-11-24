"""Are the MoE experts inside each K3 head connected to the council's agents, or to anything?

Three questions, each answered by measurement:
  1. Roster: what heads exist in the blob, and how do they relate to the 14 council agents?
  2. Identity: does any artifact or code give expert index i a name, a domain, or an owning agent?
  3. Behaviour: does the gate route different topics to different experts (specialisation), or does
     every text land on the same 1-2 experts (a collapsed mixer wearing a MoE's name)?
"""
import json
import numpy as np
from collections import Counter
from k3_2bit_study import parse_blob, head_input, BLOB, S

ver, heads, nbytes = parse_blob(BLOB)
COUNCIL = ["Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", "Flow",
           "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"]
print(f"  blob v{ver}: {len(heads)} heads")
for h in heads:
    tag = "council" if h["name"] in COUNCIL else "NOT IN COUNCIL"
    print(f"    {h['name']:14} dim={h['dim']:3} state={h['state']:3} experts={h['n_exp']:2} "
          f"buckets={h['buckets']:5}  [{tag}]")


def route(h, text):
    x, _ = head_input(h, text)
    g = h["gate_w"] @ x + h["gate_b"]
    g = np.exp(g - g.max()); g /= g.sum()
    return np.argsort(-g, kind="stable")[:2], g


tasks = json.load(open(S + "blind_prereg.json"))["tasks"]
texts = [t["text"] for t in tasks]
roster = [h for h in heads if h["name"] in COUNCIL]

print("\n  == expert utilisation across the 12 dev texts (per head: which indices get picked) ==")
print("  head          experts hit (index:count)                 distinct  max-share")
allc = Counter()
for h in roster:
    c = Counter()
    for t in texts:
        idx, _ = route(h, t)
        for i in idx:
            c[int(i)] += 1
            allc[int(i)] += 1
    tot = sum(c.values())
    top = c.most_common(1)[0]
    print(f"  {h['name']:12}  {dict(sorted(c.items()))!s:44}  {len(c):2}/{h['n_exp']:2}     "
          f"exp{top[0]}={top[1]/tot:4.0%}")

print(f"\n  pooled expert_internal hits over 14 heads: {dict(sorted(allc.items()))}")
print(f"  of {len(allc)} distinct internal indices used out of 14x8=112 slots")

print("\n  == does the same text route to the same expert index across heads? ==")
agree = 0
for t in texts:
    picks = {tuple(int(i) for i in route(h, t)[0]) for h in roster}
    agree += len(picks) == 1
print(f"  {agree}/{len(texts)} texts had identical expert picks in all 14 heads "
      f"(1 = the index means the same thing everywhere; 0 = per-head meaning)")

print("\n  == gate sharpness (is top-1 dominant?) ==")
for h in roster[:4]:
    shares = []
    for t in texts:
        _, g = route(h, t)
        shares.append(g[np.argsort(-g)[0]])
    print(f"  {h['name']:12} top-1 gate weight: mean {np.mean(shares):.3f} min {np.min(shares):.3f} "
          f"max {np.max(shares):.3f}  (1/8=0.125 uniform, top-2 => 0.5 max)")
