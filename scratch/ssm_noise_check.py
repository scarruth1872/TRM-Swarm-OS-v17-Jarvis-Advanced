
import json, collections, itertools
import numpy as np
BASE=r"F:/Development sites/TRM-Swarm-OS-v2"
d=json.load(open(BASE+"/scratch/council_ledger_snapshot.json",encoding="utf-8"))
rows=[dict(topic=p["topic"],**v) for p in d for v in p.get("votes",[])]
def noise(field, rows):
    rows=[r for r in rows if field in r]
    g=collections.defaultdict(list)
    for r in rows: g[(r["topic"],r["agent"])].append(int(r[field]))
    tot=dis=0; flipcells=0
    for k,vs in g.items():
        if len(vs)<2: continue
        if len(set(vs))>1: flipcells+=1
        for a,b in itertools.combinations(vs,2):
            tot+=1; dis+=(a!=b)
    return len(g),flipcells,dis,tot,(dis/tot if tot else float("nan"))
print("proposals",len(d),"votes",len(rows),"topics",len(set(r["topic"] for r in rows)),"pairs",len(set((r["topic"],r["agent"]) for r in rows)))
for f in ["llm_trit","vote","ssm_trit"]:
    n=noise(f,rows); print("  %-9s pairs=%d flipcells=%d pairwise-disagree=%d/%d=%.4f"%(f,n[0],n[1],n[2],n[3],n[4]))
ssm=[r for r in rows if "ssm_trit" in r]
print("ssm slice:",len(ssm),"votes;",len(set((r['topic'],r['agent']) for r in ssm)),"pairs;",len(set(r['topic'] for r in ssm)),"topics")
for f in ["llm_trit","vote"]:
    n=noise(f,ssm); print("  %-9s pairs=%d flipcells=%d pairwise-disagree=%d/%d=%.4f"%(f,n[0],n[1],n[2],n[3],n[4]))
def split_agreement(field,rows):
    rows=[r for r in rows if field in r]
    g=collections.defaultdict(list); ag=tot=0
    for r in rows: g[(r["topic"],r["agent"])].append(int(r[field]))
    rng=np.random.RandomState(0)
    for k,vs in g.items():
        if len(vs)<4: continue
        v=vs[:]; rng.shuffle(v); h=len(v)//2
        a=collections.Counter(v[:h]).most_common(1)[0][0]; b=collections.Counter(v[h:]).most_common(1)[0][0]
        ag+=(a==b); tot+=1
    return ag,tot,ag/tot
for f in ["llm_trit","vote"]:
    print("  %-9s split-half pair agreement (>=4 runs): %d/%d = %.4f"%(f,*split_agreement(f,rows)))
# per-topic label spread on the ssm slice
print("per-topic llm_trit approve share (ssm slice):")
for t in sorted(set(r["topic"] for r in ssm)):
    g=[int(r["llm_trit"]) for r in ssm if r["topic"]==t]
    print("   %-45s n=%3d approve %.2f reject %.2f abstain %.2f"%(t[:45],len(g),np.mean([x==1 for x in g]),np.mean([x==-1 for x in g]),np.mean([x==0 for x in g])))
