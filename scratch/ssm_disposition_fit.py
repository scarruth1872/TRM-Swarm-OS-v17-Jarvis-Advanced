
import json, collections, itertools, statistics, sys
import numpy as np

BASE = r"F:/Development sites/TRM-Swarm-OS-v2"
d = json.load(open(BASE+"/trm_uefi_kernel/host_council_server/data/council_ledger.json", encoding="utf-8"))
rows=[]
for p in d:
    for v in p.get("votes",[]):
        if "ssm_trit" in v:
            r=dict(topic=p["topic"], pid=p["id"], **v); rows.append(r)

def lab3(r): return int(r["llm_trit"])
print("=== DATA ===")
print("votes with ssm:", len(rows), "proposals:", len(set(r['pid'] for r in rows)),
      "topics:", len(set(r['topic'] for r in rows)),
      "pairs:", len(set((r['topic'],r['agent']) for r in rows)))
top = collections.Counter(r['topic'] for r in rows)
for t,c in top.items():
    lc = collections.Counter(lab3(r) for r in rows if r['topic']==t)
    print("  %-45s n=%3d  llm_trit dist %s" % (t[:45], c, dict(lc)))

# within-pair label consistency (noise present in THIS subset)
pg = collections.defaultdict(set)
for r in rows: pg[(r['topic'],r['agent'])].add(lab3(r))
noisy = sum(1 for k,v in pg.items() if len(v)>1)
print("pairs with >1 distinct llm_trit: %d/%d = %.3f" % (noisy, len(pg), noisy/len(pg)))
# run-to-run flip rate among pairs that have >=2 samples
import random
pairs_multi=[k for k,v in collections.Counter((r['topic'],r['agent']) for r in rows).items() if v>=2]
def flip_rate(rs):
    g=collections.defaultdict(list)
    for r in rs: g[(r['topic'],r['agent'])].append(lab3(r))
    tot=0; dis=0
    for k,vs in g.items():
        if len(vs)<2: continue
        for a,b in itertools.combinations(vs,2):
            tot+=1; dis+= (a!=b)
    return dis,tot,(dis/tot if tot else float('nan'))
print("pairwise run-to-run disagreement over %d pair-comparisons: %d/%d = %.4f" % (flip_rate(rows)[1], flip_rate(rows)[0], flip_rate(rows)[1], flip_rate(rows)[2]))

print()
print("=== BAYES CEILING ARITHMETIC ===")
p=0.19
for K in (3,2):
    print("K=%d classes: max P(agree with a fresh re-run label) = (1-p) + p/(K-1) = %.2f + %.2f/%d = %.4f" % (K,1-p,p,K-1,(1-p)+p/(K-1)))
# if 0.19 is a between-run disagreement (two noisy observations), per-run flip p0 solves 1-[(1-p0)^2+2(p0/2)^2]=0.19
import math
a,b,c = 1.5,-2.0,0.19
p0 = (-b - math.sqrt(b*b-4*a*c))/(2*a)
print("interpreting 0.19 as between-run disagreement: per-run flip p0 = %.4f -> ceiling 1-p0 = %.4f" % (p0,1-p0))

FEATS=["ssm_mean","confidence","weight"]
def X(rs): return np.array([[float(r[f]) for f in FEATS] for r in rs], float)
def Y3(rs): return np.array([lab3(r) for r in rs], int)

def onehot(y):
    M=np.zeros((len(y),3))
    for i,v in enumerate(y): M[i,{ -1:0,0:1,1:2 }[v]]=1
    return M

def fit_ridge(Xtr,Ytr,lam):
    mu=Xtr.mean(0); sd=Xtr.std(0); sd[sd==0]=1
    Z=(Xtr-mu)/sd
    Z=np.hstack([Z,np.ones((len(Z),1))])
    A=Z.T@Z + lam*np.eye(Z.shape[1]); A[-1,-1]-=lam   # don't penalise bias
    W=np.linalg.solve(A, Z.T@Ytr)
    return mu,sd,W
def pred(mu,sd,W,Xte):
    Z=(Xte-mu)/sd; Z=np.hstack([Z,np.ones((len(Z),1))])
    S=Z@W
    idx=S.argmax(1)
    return np.array([{-1:0,0:1,1:2}[v] for v in [] ]) if False else np.array([[-1,0,1][i] for i in idx])

def agree(a,b): return float(np.mean(np.array(a)==np.array(b)))

cls=[-1,0,1]
def majority(rs):
    c=collections.Counter(lab3(r) for r in rs); return c.most_common(1)[0][0], c

print()
print("=== FIT 1: ridge on [ssm_mean, confidence, weight], 5-fold CV grouped by proposal id ===")
rs=rows; Xa=X(rs); ya=Y3(rs); Yh=onehot(ya)
pids=sorted(set(r['pid'] for r in rs)); rng=np.random.RandomState(0); rng.shuffle(pids)
folds=np.array_split(np.array(pids),5)
for lam in (1.0,10.0):
    preds=np.zeros(len(rs),int)
    for f in folds:
        te=np.isin([r['pid'] for r in rs], f); tr=~te
        # foldwise: ensure all 3 classes present in train
        mu,sd,W=fit_ridge(Xa[tr],Yh[tr],lam)
        preds[te]=pred(mu,sd,W,Xa[te])
    print("  lambda %.1f  held-out agreement %.4f  (in-sample %.4f)" % (lam, agree(preds,ya), agree(pred(mu,sd,W,Xa),ya)))
    print("  pred dist", dict(collections.Counter(preds.tolist())), " true dist", dict(collections.Counter(ya.tolist())))
    # per-class recall
    for c in cls:
        m=ya==c
        if m.sum(): print("    class %+d n=%d recall %.3f"%(c,m.sum(),np.mean(preds[m]==c)))

print()
print("=== FIT 2: same ridge, LEAVE-ONE-TOPIC-OUT (6 folds) ===")
tops=sorted(set(r['topic'] for r in rs))
for lam in (1.0,10.0,100.0):
    preds=np.full(len(rs),99)
    for t in tops:
        te=np.array([r['topic']==t for r in rs]); tr=~te
        mu,sd,W=fit_ridge(Xa[tr],Yh[tr],lam)
        preds[te]=pred(mu,sd,W,Xa[te])
    per={}
    for t in tops:
        m=np.array([r['topic']==t for r in rs])
        per[t[:28]]=(round(float(np.mean(preds[m]==ya[m])),3), int(m.sum()))
    print("  lambda %5.1f  LOTO agreement %.4f   per-topic %s" % (lam, agree(preds,ya), per))
    print("     pred dist", dict(collections.Counter(preds.tolist())))

print()
print("=== FIT 3: trivial baselines ===")
c=collections.Counter(ya.tolist()); maj=c.most_common(1)[0][0]
print("  always-predict-global-majority (%+d): %.4f   (counts %s)" % (maj, c[maj]/len(ya), dict(c)))
# per-topic-majority oracle (uses test topic labels -> upper bound, not a read-out)
preds=[]
for r in rs:
    cc=collections.Counter(lab3(x) for x in rs if x['topic']==r['topic'])
    preds.append(cc.most_common(1)[0][0])
print("  per-topic-majority ORACLE (leaks test labels): %.4f  <-- not achievable by a read-out" % agree(preds,ya))
# ssm_trit as a read-out already
print("  existing ssm_trit direction vs llm_trit: %.4f" % agree([r['ssm_trit'] for r in rs], ya))
print("  existing ssm_trit vs vote:               %.4f" % agree([r['ssm_trit'] for r in rs], [r['vote'] for r in rs]))

print()
print("=== FEATURE INFORMATION (does ssm_mean carry class signal at all?) ===")
for f in FEATS:
    print("  %-12s by class:" % f, {c: round(statistics.mean(float(r[f]) for r in rs if lab3(r)==c),4) for c in cls})
# univariate threshold read-out on ssm_mean, LOTO, best threshold picked on train
best=[]
for th in sorted(set(float(r['ssm_mean']) for r in rs)):
    preds=np.array([1 if float(r['ssm_mean'])>=th else -1 for r in rs])
    pass
print("  corr(ssm_mean, llm_trit) = %.4f" % float(np.corrcoef([float(r['ssm_mean']) for r in rs], ya)[0,1]))
print("  corr(confidence, llm_trit)= %.4f" % float(np.corrcoef([float(r['confidence']) for r in rs], ya)[0,1]))
print("  corr(weight, llm_trit)    = %.4f" % float(np.corrcoef([float(r['weight']) for r in rs], ya)[0,1]))
print("  corr(ssm_steps, llm_trit) = %.4f" % float(np.corrcoef([float(r['ssm_steps']) for r in rs], ya)[0,1]))
# binary view
b=[(float(r['ssm_mean']), 1 if r['llm_trit']>0 else -1) for r in rs if r['llm_trit']!=0]
print("  binary (drop 0, n=%d) corr(ssm_mean, approve)=%.4f" % (len(b), float(np.corrcoef([x for x,_ in b],[y for _,y in b])[0,1])))
