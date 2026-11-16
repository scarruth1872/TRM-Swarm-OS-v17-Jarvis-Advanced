import sys, statistics
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2\tools')
from router_evaldata import DATA, OOD
from swarm_v2.core import domain_router as R

def probe(t):
    q = R._embed([t])[0]
    vs = R._vecs()
    s = [R._cos(q, v) for v in vs]
    o = sorted(range(len(s)), key=lambda i: -s[i])
    return o[0], s[o[0]] - s[o[1]]

REC = []
for lab, qs in DATA.items():
    for q in qs:
        top, m = probe(q)
        REC.append((lab, top, m))
for q in OOD:
    top, m = probe(q)
    REC.append((None, top, m))

def fit(pos, neg):
    if not pos or not neg:
        return None
    best = (0.0, None)
    for t in sorted({m for _, m in pos + neg}):
        tpr = sum(1 for _, m in pos if m >= t) / len(pos)
        fpr = sum(1 for _, m in neg if m >= t) / len(neg)
        j = tpr - fpr
        if j > best[0]:
            best = (j, t)
    return best[1]

print('%-36s %4s %4s %4s %7s %5s %7s %7s %7s' % 
      ('domain','prop','pos','neg','tau','acc','precis','recall','cover'))
TAU = {}
for d in range(6):
    prop = [(lab, m) for lab, top, m in REC if top == d]
    pos = [(lab, m) for lab, m in prop if lab == d]
    neg = [(lab, m) for lab, m in prop if lab != d]
    t = fit(pos, neg)
    TAU[d] = t
    if t is None:
        # No negative proposals at all: nothing can be misrouted INTO this
        # domain, so accept every proposal. tau is unbounded below.
        t = -1.0
    acc = [(lab, m) for lab, m in prop if m >= t]
    corr = sum(1 for lab, m in acc if lab == d)
    prec = corr / len(acc) if acc else 0.0
    # coverage = correctly accepted positives / all positives (NOT len(acc),
    # which includes negative proposals and made coverage exceed 1.0).
    cov = corr / len(pos) if pos else 0.0
    pa, pb = pos[0::2], pos[1::2]
    na, nb = neg[0::2], neg[1::2]
    def ev(tt, P, N):
        if tt is None: return None
        A = [(l, m) for l, m in P + N if m >= tt]
        if not A: return 0.0
        return sum(1 for l, m in A if l == d) / len(A)
    cvs = [x for x in (ev(fit(pa, na), pb, nb), ev(fit(pb, nb), pa, na)) if x is not None]
    cv = sum(cvs) / len(cvs) if cvs else float('nan')
    rec = len(pos) / len(DATA[d])
    print('%-36s %4d %4d %4d %7.4f %5d %7.3f %7.3f %7.3f   cv=%.3f' % (
          R.DOMAINS[d][0][:36], len(prop), len(pos), len(neg), t, len(acc), prec, rec, cov, cv))

glob = [(lab, top, m) for lab, top, m in REC]
gc = sum(1 for lab, top, m in glob if lab is not None and lab == top)
gn = sum(1 for lab, top, m in glob if lab is not None)
print('') 
print('global top-1 (all proposals, no abstain): %d/%d = %.3f' % (gc, gn, gc / gn))
