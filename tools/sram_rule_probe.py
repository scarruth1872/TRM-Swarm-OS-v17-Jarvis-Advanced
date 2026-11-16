import json
D = r"F:\Development sites\TRM-Swarm-OS-v2\kaggle\combined"
ch = json.load(open(D + r"\arc-agi_training2_challenges.json"))
so = json.load(open(D + r"\arc-agi_training2_solutions.json"))
ids = [t for t in list(ch)[:60] if t in so]
def rule(t):
    m = {}
    for pr in ch[t]["train"]:
        i, o = pr["input"], pr["output"]
        if len(i) != len(o) or len(i[0]) != len(o[0]):
            return None
        for r in range(len(i)):
            for c in range(len(i[0])):
                k, v = i[r][c], o[r][c]
                if m.get(k, v) != v: return None
                m[k] = v
    return m
def apply(m, g):
    return [[m.get(x, x) for x in row] for row in g]
ok = n = 0
for t in ids:
    m = rule(t)
    if m is None: continue
    n += 1
    p = apply(m, ch[t]["test"][0]["input"])
    if p == so[t][0]: ok += 1
print("tasks", len(ids))
print("rule-valid", n)
print("exact", ok)
def fh(g): return [r[::-1] for r in g]
def fv(g): return g[::-1]
def tp(g): return [list(x) for x in zip(*g)]
R = {"id": lambda g: g, "fh": fh, "fv": fv, "tp": tp,
     "r180": lambda g: fv(fh(g)),
     "r90": lambda g: fh(tp(g)),
     "r270": lambda g: fv(tp(g))}
ok2 = n2 = 0
for t in ids:
    tr = ch[t]["train"]
    good = [k for k, f in R.items()
            if all(f(pr["input"]) == pr["output"] for pr in tr)]
    if not good: continue
    n2 += 1
    p2 = R[good[0]](ch[t]["test"][0]["input"])
    if p2 == so[t][0]: ok2 += 1
print("GEOM rule-valid", n2)
print("GEOM exact", ok2)
