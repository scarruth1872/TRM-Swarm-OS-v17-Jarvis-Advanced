
import json, collections, statistics, itertools
import numpy as np
BASE=r"F:/Development sites/TRM-Swarm-OS-v2"
d=json.load(open(BASE+"/scratch/council_ledger_snapshot.json",encoding="utf-8"))
rows=[dict(topic=p["topic"],pid=p["id"],**v) for p in d for v in p.get("votes",[]) if "ssm_trit" in v]
# cell aggregation: one sample per (topic,agent) = one sample per (topic,agent), label = majority llm_trit (identical anyway)
cells=[]
for (t,a),g in itertools.groupby(sorted(rows,key=lambda r:(r["topic"],r["agent"])), key=lambda r:(r["topic"],r["agent"])):
    g=list(g); cells.append(dict(topic=t,agent=a,
        ssm_mean=statistics.mean(float(x["ssm_mean"]) for x in g),
        ssm_ambiguity=statistics.mean(float(x["ssm_ambiguity"]) for x in g),
        ssm_steps=statistics.mean(float(x["ssm_steps"]) for x in g),
        confidence=statistics.mean(float(x["confidence"]) for x in g),
        weight=statistics.mean(float(x["weight"]) for x in g),
        y=int(collections.Counter(int(x["llm_trit"]) for x in g).most_common(1)[0][0]),
        n=len(g)))
print("cells",len(cells),"topics",len(set(c["topic"] for c in cells)))
print("cell label dist",dict(collections.Counter(c["y"] for c in cells)))

def build(rs,feats):
    X=np.array([[float(r[f]) for f in feats] for r in rs]); y=np.array([r["y"] for r in rs]); return X,y
def onehot(y):
    M=np.zeros((len(y),3)); idx={-1:0,0:1,1:2}
    for i,v in enumerate(y): M[i,idx[int(v)]]=1
    return M
def ridge_fit(X,Y,lam):
    mu=X.mean(0); sd=X.std(0); sd[sd==0]=1.0
    Z=(X-mu)/sd; Z=np.hstack([Z,np.ones((len(Z),1))])
    A=Z.T@Z+lam*np.eye(Z.shape[1]); A[-1,-1]-=lam
    return mu,sd,np.linalg.solve(A,Z.T@Y)
def predict(m,X):
    mu,sd,W=m; Z=(X-mu)/sd; Z=np.hstack([Z,np.ones((len(Z),1))]); s=Z@W
    return np.array([[-1,0,1][i] for i in s.argmax(1)])
def loto(rs,feats,lam=10.0):
    X,y=build(rs,feats); Y=onehot(y); preds=np.zeros(len(y),int)
    for t in sorted(set(r["topic"] for r in rs)):
        te=np.array([r["topic"]==t for r in rs]); tr=~te
        if len(set(y[tr]))<3:  # keep numeric stability if class missing
            pass
        preds[te]=predict(ridge_fit(X[tr],Y[tr],lam),X[te])
    return preds,y
def acc(p,y): return float(np.mean(np.array(p)==np.array(y)))

cells=[c for c in cells]
C=sorted(cells,key=lambda c:(c["topic"],c["agent"]))
print()
print("=== LOTO ABLATION on %d cells / %d topics (ridge, lambda 10) ===" % (len(C),len(set(c["topic"] for c in C))))
subsets={
 "ssm_mean only":["ssm_mean"],
 "confidence only":["confidence"],
 "weight only":["weight"],
 "ssm_mean+confidence":["ssm_mean","confidence"],
 "ssm_mean+confidence+weight (spec'd)":["ssm_mean","confidence","weight"],
 "all ssm fields":["ssm_mean","ssm_ambiguity","ssm_steps"],
}
for name,f in subsets.items():
    p,y=loto(C,f)
    print("  %-36s LOTO %.4f   pred dist %s" % (name,acc(p,y),dict(collections.Counter(p.tolist()))))
c=collections.Counter(x["y"] for x in C); maj=c.most_common(1)[0][0]
print("  %-36s      %.4f   (majority class %+d, %d/%d)" % ("always-majority baseline",c[maj]/len(C),maj,c[maj],len(C)))

# vote level too, for comparability with earlier 0.458/0.474 numbers
print()
print("=== same at VOTE level (%d ssm votes), LOTO ===" % len(rows))
V=[dict(r,y=int(r["llm_trit"])) for r in rows]
for name,f in [("ssm_mean only",["ssm_mean"]),("ssm_mean+confidence+weight",["ssm_mean","confidence","weight"])]:
    p,y=loto(V,f); print("  %-36s LOTO %.4f"%(name,acc(p,y)))
cv=collections.Counter(x["y"] for x in V); m=cv.most_common(1)[0][0]
print("  %-36s      %.4f" % ("always-majority baseline",cv[m]/len(V)))

# how much of LOTO 0.79 is just "predict approve"? and topic-level structure
print()
print("=== STRUCTURE ===")
for t in sorted(set(c["topic"] for c in C)):
    g=[c for c in C if c["topic"]==t]
    print("  %-45s n=%2d approve %2d  (%.2f)  mean ssm_mean %+.4f mean conf %.3f" %
          (t[:45],len(g),sum(1 for c in g if c["y"]==1),np.mean([c["y"]==1 for c in g]),np.mean([c["ssm_mean"] for c in g]),np.mean([c["confidence"] for c in g])))
# within-topic vs between-topic variance of label and of features
ys=np.array([c["y"] for c in C],float)
gm=ys.mean(); tot=((ys-gm)**2).sum()
within=sum(((np.array([c["y"] for c in C if c["topic"]==t],float)-np.mean([c["y"] for c in C if c["topic"]==t]))**2).sum() for t in set(c["topic"] for c in C))
print("  label variance explained by topic: %.4f (within-topic SS %.3f / total %.3f)"%(1-within/tot,within,tot))
# permutation test on the spec'd LOTO
p,y=loto(C,["ssm_mean","confidence","weight"]); obs=acc(p,y)
rng=np.random.RandomState(1); cnt=0; B=2000
for _ in range(B):
    perm=rng.permutation([c["y"] for c in C])
    Cp=[dict(c,y=int(v)) for c,v in zip(C,perm)]
    if acc(loto(Cp,["ssm_mean","confidence","weight"])[0],perm)>=obs: cnt+=1
print("  permutation p-value (label shuffled within cells, %d perms): %.3f"%(B,(cnt+1)/(B+1)))
# leave-one-PAIR-out (i.e. random cell split) for contrast
from numpy.random import RandomState
rng=RandomState(7); idx=rng.permutation(len(C)); k=5; folds=np.array_split(idx,k)
pred=np.zeros(len(C),int); yy=np.array([c["y"] for c in C])
X=np.array([[c[f] for f in ["ssm_mean","confidence","weight"]] for c in C])
for f in folds:
    tr=np.setdiff1d(idx,f)
    pred[f]=predict(ridge_fit(X[tr],onehot(yy[tr]),10.0),X[f])
print("  cell-level random 5-fold (leaks topic+agent): %.4f   <- not an honest split" % acc(pred,yy))
